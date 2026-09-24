export function countConverterInputLines(value: string) {
  return value
    .split(/\r\n|\n|\r/)
    .filter((line) => line.trim().length > 0)
    .length;
}
