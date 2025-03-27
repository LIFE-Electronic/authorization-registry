export function arrayValueToDisplay(value: string[]): string {
  return value.map((value) => `'${value}'`).join(", ");
}
