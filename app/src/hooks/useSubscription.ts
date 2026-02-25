import { useEffect, useState, useRef, useCallback } from "react";

import { getWsInstance } from "@/utils/constants";
import { decodeProtoMessage } from "@/utils/protoHelpers";

const RECONNECTION_DELAY = 3000; // 3 seconds
const MAX_RECONNECTION_ATTEMPTS = 10;

export default function useSubscription<T>(
  path: string,
  lookupKey: string,
  payload: Record<string, any>,
  maintainPreviousMessages = true,
  disabled?: boolean,
): {
  messages: T[];
} {
  const [messages, setMessages] = useState<T[]>([]);

  const wsRef = useRef(getWsInstance());
  const payloadSentRef = useRef(false);
  const reconnectionTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const subscribedRef = useRef(false);

  const handleMessage = useCallback(
    async function (event: MessageEvent) {
      if (disabled) return;

      if (event.data instanceof Blob) {
        try {
          const decodedData: T = await decodeProtoMessage<T>(
            path,
            lookupKey,
            await event.data.arrayBuffer(),
          );
          if (maintainPreviousMessages)
            setMessages((prevMessages) => [decodedData, ...prevMessages]);
          else setMessages([decodedData]);
        } catch (error) {
          if (!(error instanceof RangeError)) {
            console.error("Error decoding protobuf:", error);
          }
        }
      }
    },
    [lookupKey, path, disabled],
  );

  useEffect(() => {
    if (disabled) return;

    const ws = wsRef.current;
    if (ws.readyState === ws.OPEN) {
      sendInitialPayload();
    }

    function sendInitialPayload() {
      const ws = wsRef.current;

      if (ws.readyState === ws.OPEN && !payloadSentRef.current) {
        ws.send(JSON.stringify(payload));
        payloadSentRef.current = true;
        // subscribe to message
        subscribedRef.current = true;
        ws.addEventListener("message", handleMessage);
      }
    }

    function handleOpen() {
      if (disabled) return;

      const ws = wsRef.current;
      if (ws.readyState === ws.OPEN) {
        sendInitialPayload();
      }
    }

    function reconnect() {
      if (disabled) return;

      // reconnection logic
      if (reconnectionTimeoutRef.current)
        clearTimeout(reconnectionTimeoutRef.current);

      for (let i = 1; i <= MAX_RECONNECTION_ATTEMPTS; i++) {
        reconnectionTimeoutRef.current = setTimeout(
          () => {
            if (disabled) return;

            if (wsRef.current.readyState !== ws.OPEN) {
              wsRef.current = getWsInstance(true);
            } else if (wsRef.current.readyState === ws.OPEN) {
              handleOpen();
              clearTimeout(reconnectionTimeoutRef.current!);
              reconnectionTimeoutRef.current = null;
            }
          },
          RECONNECTION_DELAY * (i + 1), // exponential backoff
        );
      }
    }

    function handleClose() {
      if (disabled) return;

      payloadSentRef.current = false;
      reconnect();
    }

    ws.addEventListener("open", handleOpen);
    ws.addEventListener("close", handleClose);
    ws.addEventListener("error", (event) => {
      if (disabled) return;

      console.error("WebSocket error:", event);
      reconnect();
    });

    return () => {
      ws.removeEventListener("open", handleOpen);
      ws.removeEventListener("close", handleClose);
      ws.removeEventListener("error", () => {
        reconnect();
      });
    };
  }, [handleMessage, payload, disabled]);

  // Handle incoming messages
  useEffect(() => {
    if (disabled) return;

    const ws = wsRef.current;
    subscribedRef.current = true;

    ws.addEventListener("message", handleMessage);

    return function () {
      ws.removeEventListener("message", handleMessage);
    };
  }, [handleMessage, lookupKey, path, disabled]);

  return { messages };
}
