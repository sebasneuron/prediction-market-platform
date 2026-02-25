export type Order = {
  created_at: string;
  filled_quantity: string;
  id: string;
  market_id: string;
  outcome: "yes" | "no" | "settled";
  price: string;
  quantity: string;
  side: "buy" | "sell";
  status: OrderCategory;
  updated_at: string;
  user_id: string;
  order_type: "limit" | "market";
};

export type OrderCategory =
  | "open"
  | "cancelled"
  | "filled"
  | "expired"
  | "pending_update"
  | "pending_cancel"
  | "all";

export type PageInfoServiceAPi = {
  page: number;
  page_size: number;
  total_items: number;
  total_pages: number;
};
