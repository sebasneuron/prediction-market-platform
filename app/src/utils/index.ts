export function formatDate<T extends string | number>(date: T): string {
  const dt = new Date(date).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  return dt;
}

export function formatPriceString(
  price: number | string,
  precision: number = 2,
): string {
  const formatter = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: precision,
    maximumFractionDigits: precision,
  });

  return formatter.format(Number(price || 0));
}
