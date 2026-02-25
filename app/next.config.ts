import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  experimental: {
    optimizePackageImports: ["@chakra-ui/react"],
  },
  output: "standalone",
  env: {
    NEXT_PUBLIC_GOOGLE_CLIENT_ID: process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID,
    NEXT_PUBLIC_SERVICE_API_URL: process.env.NEXT_PUBLIC_SERVICE_API_URL,
    NEXT_PUBLIC_GRPC_API_URL: process.env.NEXT_PUBLIC_GRPC_API_URL,
    NEXT_PUBLIC_WS_API_URL: process.env.NEXT_PUBLIC_WS_API_URL,
  },
};

export default nextConfig;
