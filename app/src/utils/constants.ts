// global websocket instance
let instance: WebSocket | null = null;

export function getWsInstance(forceReconnect?: boolean) {
  if (forceReconnect && instance) {
    instance.close();
    instance = null;
  }
  if (!instance) {
    instance = new WebSocket(process.env.NEXT_PUBLIC_WS_API_URL!);
    instance.onopen = () => {
      console.log("WebSocket connection established.");
    };
  }
  return instance;
}
