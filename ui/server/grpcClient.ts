import * as grpc from "@grpc/grpc-js";
import * as protoLoader from "@grpc/proto-loader";
import path from "path";

const PROTO_DIR = path.resolve(import.meta.dirname, "../../proto");

const PROTO_FILES = [
  "issuetracker/v1/component.proto",
  "issuetracker/v1/issue.proto",
  "issuetracker/v1/comment.proto",
  "issuetracker/v1/hotlist.proto",
  "issuetracker/v1/search.proto",
  "issuetracker/v1/event_log.proto",
  "issuetracker/v1/acl.proto",
];

const packageDef = protoLoader.loadSync(PROTO_FILES, {
  keepCase: false,
  longs: Number,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [PROTO_DIR],
});

const proto = grpc.loadPackageDefinition(packageDef);

interface PackageV1 {
  ComponentService: grpc.ServiceClientConstructor;
  IssueService: grpc.ServiceClientConstructor;
  CommentService: grpc.ServiceClientConstructor;
  HotlistService: grpc.ServiceClientConstructor;
  SearchService: grpc.ServiceClientConstructor;
  EventLogService: grpc.ServiceClientConstructor;
  AclService: grpc.ServiceClientConstructor;
}

function getPackage(): PackageV1 {
  const root = proto["issuetracker"] as grpc.GrpcObject;
  return root["v1"] as unknown as PackageV1;
}

export interface GrpcClients {
  component: grpc.Client;
  issue: grpc.Client;
  comment: grpc.Client;
  hotlist: grpc.Client;
  search: grpc.Client;
  eventLog: grpc.Client;
  acl: grpc.Client;
}

export function createClients(serverAddr: string): GrpcClients {
  const pkg = getPackage();
  const creds = grpc.credentials.createInsecure();
  return {
    component: new pkg.ComponentService(serverAddr, creds),
    issue: new pkg.IssueService(serverAddr, creds),
    comment: new pkg.CommentService(serverAddr, creds),
    hotlist: new pkg.HotlistService(serverAddr, creds),
    search: new pkg.SearchService(serverAddr, creds),
    eventLog: new pkg.EventLogService(serverAddr, creds),
    acl: new pkg.AclService(serverAddr, creds),
  };
}

function callOnce<T>(client: grpc.Client, method: string, request: unknown, metadata?: grpc.Metadata): Promise<T> {
  return new Promise((resolve, reject) => {
    const fn = (client as Record<string, Function>)[method];
    if (!fn) {
      reject(new Error(`Unknown method: ${method}`));
      return;
    }
    const args: unknown[] = [request];
    if (metadata) {
      args.push(metadata);
    }
    args.push((err: grpc.ServiceError | null, response: T) => {
      if (err) reject(err);
      else resolve(response);
    });
    fn.apply(client, args);
  });
}

const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 100;

export async function call<T>(client: grpc.Client, method: string, request: unknown, metadata?: grpc.Metadata): Promise<T> {
  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    try {
      return await callOnce<T>(client, method, request, metadata);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      const isTransactionConflict = msg.includes("cannot start a transaction within a transaction")
        || msg.includes("database is locked");
      if (isTransactionConflict && attempt < MAX_RETRIES - 1) {
        await new Promise((r) => setTimeout(r, RETRY_DELAY_MS * (attempt + 1)));
        continue;
      }
      throw err;
    }
  }
  throw new Error("unreachable");
}

export function metadataFromUserId(userId: string | undefined): grpc.Metadata | undefined {
  if (!userId) return undefined;
  const meta = new grpc.Metadata();
  meta.set("x-user-id", userId);
  return meta;
}
