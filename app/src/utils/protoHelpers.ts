import { load, Type } from "protobufjs";

const protoMap = new Map<string, Type>();

async function loadProtoFile(filePath: string, lookupKey: string) {
  const cacheKey = `${filePath}:${lookupKey}`;

  if (protoMap.has(cacheKey)) {
    return protoMap.get(cacheKey)!;
  }

  if (filePath.endsWith(".proto")) {
    const root = await load(filePath);
    const messageType = root.lookupType(lookupKey);
    protoMap.set(cacheKey, messageType);

    return messageType;
  } else {
    throw new Error("Invalid file type. Only .proto files are supported.");
  }
}

export async function encodeProtoMessage(
  filePath: string,
  lookupKey: string,
  message: object,
): Promise<Uint8Array> {
  const messageType = await loadProtoFile(filePath, lookupKey);
  const errMsg = messageType.verify(message);
  if (errMsg) {
    throw new Error(errMsg);
  }
  const buffer = messageType.encode(message).finish();
  return buffer;
}

export async function decodeProtoMessage<T>(
  filePath: string,
  lookupKey: string,
  buffer: ArrayBufferLike,
): Promise<T> {
  const messageType = await loadProtoFile(filePath, lookupKey);
  const uintData = new Uint8Array(buffer);
  const message = messageType.decode(uintData);
  const verified = messageType.verify(uintData);
  if (verified) throw new Error("Invalid data");
  const obj = messageType.toObject(message, {
    longs: String,
    enums: String,
    bytes: String,
  }) as unknown as T;
  return obj;
}
