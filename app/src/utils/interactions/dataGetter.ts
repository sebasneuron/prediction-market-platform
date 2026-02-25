import axios from "axios";
import jsCookies from "js-cookie";

import { marketServiceClient, priceServiceClient } from "../grpc/clients";
import {
  GetUserHoldingsResponse,
  GetUserMetadataResponse,
  GetUserOrdersPaginatedResponse,
  GetUserResponse,
  GetUserTradesResponse,
} from "../types/api";
import { OrderCategory } from "../types";

import { Timeframe } from "@/generated/grpc_service_types/common";
import {
  GetMarketTradesResponse,
  MarketStatus,
} from "@/generated/grpc_service_types/markets";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class MarketGetters {
  static async getMarketData(
    page: number,
    pageSize: number,
    marketStatus: MarketStatus,
  ) {
    try {
      const data = await marketServiceClient.getMarketData({
        pageRequest: {
          page,
          pageSize,
        },
        marketStatus,
      });
      return data.response.markets;
    } catch (error: any) {
      console.error("Error fetching market data:", error);
      return [];
    }
  }

  static async getMarketById(marketId: string) {
    try {
      const { response } = await marketServiceClient.getMarketById({
        marketId,
      });
      return response;
    } catch (error: any) {
      console.log("Failed to get market due to ", error);
      return null;
    }
  }

  static async getOrderBook(marketId: string, depth: number = 10) {
    try {
      const { response } = await marketServiceClient.getMarketBook({
        depth,
        marketId,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get order book: ", error);
      return null;
    }
  }

  static async getTopTenHolders(marketId: string) {
    try {
      const { response } = await marketServiceClient.getTopHolders({
        marketId,
      });
      return response.topHolders;
    } catch (error: any) {
      console.error("Failed to get top ten holders: ", error);
      return [];
    }
  }

  static async getMarketTrades({
    marketId,
    page,
    pageSize,
  }: {
    marketId: string;
    page: number;
    pageSize: number;
  }): Promise<GetMarketTradesResponse> {
    try {
      const { response } = await marketServiceClient.getMarketTrades({
        marketId,
        pageRequest: {
          page,
          pageSize,
        },
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get market trades: ", error);
      return {
        trades: [],
        marketId: "",
        pageInfo: {
          page: 0,
          pageSize: 0,
          totalPages: 0,
          totalItems: 0,
        },
      };
    }
  }
}

export class UserGetters {
  static async getUserData() {
    try {
      const { data, status } = await axios.get<GetUserResponse>(
        `${BASE_URL}/user/profile`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      if (status !== 200) {
        throw new Error("Failed to fetch user data");
      }
      return data;
    } catch (e: any) {
      console.log("Error fetching user data:", e);
      return null;
    }
  }

  static async getUserMetadata() {
    try {
      const { data } = await axios.get<GetUserMetadataResponse>(
        `${BASE_URL}/user/metadata`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user metadata: ", error);
      return null;
    }
  }

  static async getUserTrades(
    page: number,
    pageSize: number,
  ): Promise<GetUserTradesResponse> {
    try {
      const { data } = await axios.get<GetUserTradesResponse>(
        `${BASE_URL}/user/trades?page=${page}&pageSize=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user trades: ", error);
      return {
        data: {
          trades: [],
          page_info: {
            page: 0,
            page_size: 0,
            total_items: 0,
            total_pages: 0,
          },
        },
      };
    }
  }

  static async getUserHoldings(
    page: number,
    pageSize: number,
  ): Promise<GetUserHoldingsResponse> {
    try {
      const { data } = await axios.get<GetUserHoldingsResponse>(
        `${BASE_URL}/user/holdings?page=${page}&pageSize=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      return data;
    } catch (error: any) {
      console.error("Failed to get user holdings: ", error);
      return {
        data: {
          holdings: [],
          page_info: {
            page: 0,
            page_size: 0,
            total_items: 0,
            total_pages: 0,
          },
        },
      };
    }
  }
}

export class OrderGetters {
  static async getUserOrdersPaginated(page: number, pageSize: number) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get?page=${page}&page_size=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return { orders: [], page: 0, page_size: 0 };
    }
  }

  static async getUserOrdersByMarket(
    marketId: string,
    page: number,
    pageSize: number,
    orderType: OrderCategory = "all",
  ) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get/${marketId}?page=${page}&page_size=${pageSize}&status=${orderType}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return {
        orders: [],
        page: 0,
        page_size: 0,
        holdings: { no: "0", yes: "0" },
        total_pages: 0,
      };
    }
  }
}

export class ChartGetters {
  static async getChartDataWithinTimeRange(
    marketId: string,
    timeframe: Timeframe,
  ) {
    try {
      const { response } = await priceServiceClient.getPriceDataWithinInterval({
        marketId,
        timeframe,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get chart data: ", error);
      return {
        marketId: "",
        priceData: [],
      };
    }
  }
}
