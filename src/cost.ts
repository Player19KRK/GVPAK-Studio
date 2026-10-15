export type Currency = "USD" | "CNY";
export type ModelConfig = {
  id: string;
  name: string;
  resolutions: string[];
  ratios: string[];
  inputUsdPerMillion: number;
  textUsdPerMillion: number;
  imageUsdPerMillion: number;
  imageUsd: Record<string, number | null | undefined>;
  source: string;
  checkedAt: string;
  note: string;
};
export function estimate(
  model: ModelConfig,
  resolution: string,
  input: number,
  output: number,
  count = 1,
): number | null {
  if (
    ![input, output, count].every(Number.isFinite) ||
    input < 0 ||
    output < 0 ||
    !Number.isInteger(count) ||
    count < 1
  )
    return null;
  const image = model.imageUsd[resolution];
  if (image == null) return null;
  return (
    (image +
      (input * model.inputUsdPerMillion) / 1e6 +
      (output * model.textUsdPerMillion) / 1e6) *
    count
  );
}
export function money(
  usd: number | null,
  currency: Currency,
  rate: number,
): string {
  if (
    usd == null ||
    !Number.isFinite(usd) ||
    (currency === "CNY" && (!Number.isFinite(rate) || rate <= 0))
  )
    return "Unavailable";
  return `${currency === "CNY" ? "¥" : "$"}${(usd * (currency === "CNY" ? rate : 1)).toFixed(4)}`;
}

