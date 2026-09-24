export class ConversionRequestGuard {
  private revision = 0;

  begin() {
    this.revision += 1;
    return this.revision;
  }

  invalidate() {
    this.revision += 1;
  }

  isCurrent(revision: number) {
    return revision === this.revision;
  }
}
