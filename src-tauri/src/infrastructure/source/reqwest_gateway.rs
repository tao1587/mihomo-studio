use std::time::Duration;

use futures_util::StreamExt;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    redirect, Client,
};

use crate::{
    application::ports::{
        SubscriptionFetchResult, SubscriptionGateway, SubscriptionGatewayFactory,
    },
    domain::source::{
        has_credentials, is_http_url, is_loopback_url, FetchRoute, SourceError, SubscriptionUrl,
        MAX_SOURCE_BYTES,
    },
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_REDIRECTS: usize = 5;

pub struct ReqwestSubscriptionGateway {
    client: Client,
}

impl ReqwestSubscriptionGateway {
    fn new(route: FetchRoute) -> Result<Self, SourceError> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let mut builder =
            Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .redirect(redirect::Policy::custom(|attempt| {
                    if attempt.previous().len() > MAX_REDIRECTS {
                        return attempt.error("redirect limit exceeded");
                    }

                    let next = attempt.url();
                    if !is_http_url(next) || has_credentials(next) {
                        return attempt.error("unsafe redirect target");
                    }

                    if let Some(previous) = attempt.previous().last() {
                        if previous.scheme() == "https" && next.scheme() == "http" {
                            return attempt.error("https downgrade blocked");
                        }
                        if !is_loopback_url(previous) && is_loopback_url(next) {
                            return attempt.error("remote to loopback redirect blocked");
                        }
                    }

                    attempt.follow()
                }));

        if route == FetchRoute::Direct {
            builder = builder.no_proxy();
        }

        let client = builder
            .build()
            .map_err(|_| SourceError::ClientInitialization)?;
        Ok(Self { client })
    }

    async fn fetch_once(
        &self,
        source: &SubscriptionUrl,
        user_agent: &str,
    ) -> Result<SubscriptionFetchResult, SourceError> {
        let response = self
            .client
            .get(source.as_url().clone())
            .header(USER_AGENT, user_agent)
            .header(
                ACCEPT,
                "text/yaml, application/yaml, text/plain, application/octet-stream;q=0.8, */*;q=0.2",
            )
            .send()
            .await
            .map_err(classify_request_error)?;

        if !response.status().is_success() {
            return Ok(SubscriptionFetchResult::HttpStatus(
                response.status().as_u16(),
            ));
        }

        if response
            .content_length()
            .is_some_and(|length| length > MAX_SOURCE_BYTES as u64)
        {
            return Err(SourceError::ResponseTooLarge);
        }

        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(classify_request_error)?;
            if bytes.len().saturating_add(chunk.len()) > MAX_SOURCE_BYTES {
                return Err(SourceError::ResponseTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }

        Ok(SubscriptionFetchResult::Body(bytes))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReqwestSubscriptionGatewayFactory;

impl SubscriptionGatewayFactory for ReqwestSubscriptionGatewayFactory {
    fn create(&self, route: FetchRoute) -> Result<Box<dyn SubscriptionGateway>, SourceError> {
        ReqwestSubscriptionGateway::new(route)
            .map(|gateway| Box::new(gateway) as Box<dyn SubscriptionGateway>)
    }
}

impl SubscriptionGateway for ReqwestSubscriptionGateway {
    fn fetch<'a>(
        &'a self,
        source: &'a SubscriptionUrl,
        user_agent: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<SubscriptionFetchResult, SourceError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.fetch_once(source, user_agent))
    }
}

fn classify_request_error(error: reqwest::Error) -> SourceError {
    if error.is_timeout() {
        SourceError::RequestTimeout
    } else if error.is_connect() {
        SourceError::ConnectionFailed
    } else if error.is_redirect() {
        SourceError::RedirectRejected
    } else if error.is_body() || error.is_decode() {
        SourceError::ResponseReadFailed
    } else {
        SourceError::RequestFailed
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use base64::{engine::general_purpose, Engine as _};

    use crate::{
        application::source::inspect_subscription, domain::source::SubscriptionInspectionRequest,
    };

    use super::*;

    #[test]
    #[ignore = "requires loopback socket permission"]
    fn retries_compatible_user_agents_until_a_subscription_is_recognized() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        let address = listener.local_addr().expect("fixture address");
        let encoded =
            general_purpose::STANDARD.encode("vless://fixture-id@example.invalid:443#Fixture");

        let server = thread::spawn(move || {
            for index in 0..3 {
                let (mut stream, _) = listener.accept().expect("fixture request");
                let mut request = [0_u8; 4096];
                let length = stream.read(&mut request).expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..length]).to_ascii_lowercase();

                let (status, body) = match index {
                    0 => {
                        assert!(request.contains("user-agent: mihomo/1.19"));
                        ("403 Forbidden", String::new())
                    }
                    1 => {
                        assert!(request.contains("user-agent: clash.meta"));
                        ("200 OK", "not-a-subscription".to_string())
                    }
                    _ => {
                        assert!(request.contains("user-agent: clash-verge"));
                        ("200 OK", encoded.clone())
                    }
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write fixture response");
            }
        });

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("fixture runtime");
        let result = runtime
            .block_on(inspect_subscription(
                &ReqwestSubscriptionGatewayFactory,
                SubscriptionInspectionRequest {
                    url: format!("http://{address}/fixture-subscription"),
                    fetch_route: "direct".into(),
                },
            ))
            .expect("third User-Agent should recognize fixture");
        server.join().expect("fixture server completes");

        assert_eq!(result.node_count, 1);
        assert_eq!(result.source_format, "base64-uri-list");
        assert_eq!(result.user_agent.as_deref(), Some("Clash Verge"));
        assert!(result.safe_label.ends_with("/••••••"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("前 2 个")));
    }
}
