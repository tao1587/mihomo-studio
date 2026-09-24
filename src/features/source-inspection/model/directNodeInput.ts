export function countNonEmptyNodeLines(value: string): number {
  return value.split(/\r\n|\r|\n/).filter((line) => line.trim().length > 0).length;
}
