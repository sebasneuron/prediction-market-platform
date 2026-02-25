import axios, { AxiosError } from "axios";
import jsCookies from "js-cookie";

import { LoginResponse } from "../types/api";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class UserAuthActions {
  static async handleSignInWithGoogle({ id_token }: { id_token: string }) {
    const { data, status } = await axios.post(`${BASE_URL}/login`, {
      id_token,
    });

    if (status != 200) throw new Error(data.error);
    return data as LoginResponse;
  }
}

export class MarketActions {
  static async createLimitOrder(reqPayload: {
    market_id: string;
    price: number;
    quantity: number;
    side: "buy" | "sell";
    outcome_side: "yes" | "no";
  }) {
    try {
      await axios.post(`${BASE_URL}/user/orders/create/limit`, reqPayload, {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${TOKEN}`,
        },
      });
    } catch (error: any) {
      console.error("Error creating limit order:", error);
      if (error instanceof AxiosError) {
        console.log("Axios error details:", error.response?.data);
        throw new Error(
          error.response?.data?.error || "Failed to create limit order",
        );
      }
      throw new Error("Failed to create limit order");
    }
  }

  static async updateOrder(payload: {
    order_id: string;
    new_quantity: number;
    new_price: number;
  }) {
    try {
      const { data } = await axios.patch(
        `${BASE_URL}/user/orders/update`,
        payload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Error updating order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to update order",
        );
      }
      throw new Error("Failed to update order");
    }
  }

  static async cancelOrder(orderId: string) {
    try {
      const { data } = await axios.delete(
        `${BASE_URL}/user/orders/cancel/${orderId}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Error canceling order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to cancel order",
        );
      }
      throw new Error("Failed to cancel order");
    }
  }

  static async createMarketOrder(reqPayload: {
    market_id: string;
    price: number;
    outcome: "yes" | "no";
    side: "buy" | "sell";
  }) {
    try {
      await axios.post(`${BASE_URL}/user/orders/create/market`, reqPayload, {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${TOKEN}`,
        },
      });
    } catch (error: any) {
      console.error("Error creating market order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create market order",
        );
      }
      throw new Error("Failed to create market order");
    }
  }
}
