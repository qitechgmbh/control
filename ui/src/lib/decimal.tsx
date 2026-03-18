export function roundToDecimals(num: number, decimals: number): string {
  let output = num.toFixed(decimals);

  // if the number is between 0 and < -0.(decimals-1)1 omit the - sign
  if (num <= Math.pow(10, -decimals) && num >= -Math.pow(10, -decimals)) {
    output = output.replace("-", "​");
  }

  return output;
}

/// same as roundToDecimals
/// but round from 359.5 to 0.0
export function roundDegreesToDecimals(num: number, decimals: number): string {
  const threshold = 360 - Math.pow(10, -decimals) / 2;
  const output = num >= threshold ? 0 : num;

  return roundToDecimals(output, decimals);
}
