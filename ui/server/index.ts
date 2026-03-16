import express from "express";
import { createClients, call, metadataFromUserId, type GrpcClients } from "./grpcClient.js";

const GRPC_SERVER = process.env["IT_SERVER_ADDR"] ?? "localhost:50051";
const PORT = Number(process.env["API_PORT"] ?? "3001");

const app = express();
app.use(express.json());

let clients: GrpcClients;

function getUserId(req: express.Request): string | undefined {
  return req.headers["x-user-id"] as string | undefined;
}

// --- Components ---

app.get("/api/components", async (_req, res) => {
  try {
    const result = await call(clients.component, "listComponents", { pageSize: 100 });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/components", async (req, res) => {
  try {
    const result = await call(clients.component, "createComponent", {
      name: req.body.name,
      description: req.body.description,
      parentId: req.body.parentId,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/components/:id", async (req, res) => {
  try {
    const result = await call(clients.component, "getComponent", {
      componentId: Number(req.params["id"]),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.patch("/api/components/:id", async (req, res) => {
  try {
    const result = await call(clients.component, "updateComponent", {
      componentId: Number(req.params["id"]),
      name: req.body.name,
      description: req.body.description,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.delete("/api/components/:id", async (req, res) => {
  try {
    const result = await call(clients.component, "deleteComponent", {
      componentId: Number(req.params["id"]),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Issues ---

app.get("/api/issues", async (req, res) => {
  try {
    const result = await call(clients.issue, "listIssues", {
      componentId: req.query["componentId"] ? Number(req.query["componentId"]) : undefined,
      pageSize: Number(req.query["pageSize"] ?? "50"),
      pageToken: req.query["pageToken"] ?? "",
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/issues", async (req, res) => {
  try {
    const result = await call(clients.issue, "createIssue", req.body);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/issues/:id", async (req, res) => {
  try {
    const result = await call(clients.issue, "getIssue", {
      issueId: Number(req.params["id"]),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.patch("/api/issues/:id", async (req, res) => {
  try {
    const result = await call(clients.issue, "updateIssue", {
      issueId: Number(req.params["id"]),
      ...req.body,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Issue Relationships ---

app.post("/api/issues/:id/parent", async (req, res) => {
  try {
    const result = await call(clients.issue, "addParent", {
      childId: Number(req.params["id"]),
      parentId: req.body.parentId,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/issues/:id/parents", async (req, res) => {
  try {
    const result = await call(clients.issue, "listParents", {
      issueId: Number(req.params["id"]),
    });
    // issueId matches proto field name
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/issues/:id/children", async (req, res) => {
  try {
    const result = await call(clients.issue, "listChildren", {
      issueId: Number(req.params["id"]),
    });
    // issueId matches proto field name
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/issues/:id/blocking", async (req, res) => {
  try {
    const result = await call(clients.issue, "addBlocking", {
      blockingId: Number(req.params["id"]),
      blockedId: req.body.blockedId,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/issues/:id/duplicate", async (req, res) => {
  try {
    const result = await call(clients.issue, "markDuplicate", {
      issueId: Number(req.params["id"]),
      canonicalId: req.body.canonicalId,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Comments ---

app.get("/api/issues/:issueId/comments", async (req, res) => {
  try {
    const meta = metadataFromUserId(getUserId(req));
    const result = await call(clients.comment, "listComments", {
      issueId: Number(req.params["issueId"]),
      pageSize: 100,
    }, meta);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/issues/:issueId/comments", async (req, res) => {
  try {
    const meta = metadataFromUserId(getUserId(req));
    const result = await call(clients.comment, "createComment", {
      issueId: Number(req.params["issueId"]),
      body: req.body.body,
      author: req.body.author,
    }, meta);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.patch("/api/comments/:commentId", async (req, res) => {
  try {
    const meta = metadataFromUserId(getUserId(req));
    const result = await call(clients.comment, "updateComment", {
      commentId: Number(req.params["commentId"]),
      body: req.body.body,
    }, meta);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/comments/:commentId/hide", async (req, res) => {
  try {
    const meta = metadataFromUserId(getUserId(req));
    const result = await call(clients.comment, "hideComment", {
      commentId: Number(req.params["commentId"]),
      hidden: req.body.hidden ?? true,
    }, meta);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/comments/:commentId/revisions", async (req, res) => {
  try {
    const meta = metadataFromUserId(getUserId(req));
    const result = await call(clients.comment, "listCommentRevisions", {
      commentId: Number(req.params["commentId"]),
      pageSize: 50,
    }, meta);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Hotlists ---

app.get("/api/hotlists", async (_req, res) => {
  try {
    const result = await call(clients.hotlist, "listHotlists", { pageSize: 100 });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/hotlists", async (req, res) => {
  try {
    const result = await call(clients.hotlist, "createHotlist", req.body);
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/hotlists/:id", async (req, res) => {
  try {
    const result = await call(clients.hotlist, "getHotlist", {
      hotlistId: Number(req.params["id"]),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.get("/api/hotlists/:id/issues", async (req, res) => {
  try {
    const result = await call(clients.hotlist, "listIssues", {
      hotlistId: Number(req.params["id"]),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

app.post("/api/hotlists/:id/issues", async (req, res) => {
  try {
    const result = await call(clients.hotlist, "addIssue", {
      hotlistId: Number(req.params["id"]),
      issueId: req.body.issueId,
      position: req.body.position,
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Search ---

app.get("/api/search", async (req, res) => {
  try {
    const result = await call(clients.search, "searchIssues", {
      query: (req.query["q"] as string) ?? "",
      orderBy: (req.query["orderBy"] as string) ?? "",
      orderDirection: (req.query["orderDir"] as string) ?? "",
      pageSize: Number(req.query["pageSize"] ?? "50"),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Events ---

app.get("/api/events", async (req, res) => {
  try {
    const result = await call(clients.eventLog, "listEvents", {
      entityType: (req.query["entityType"] as string) ?? "",
      entityId: req.query["entityId"] ? Number(req.query["entityId"]) : undefined,
      pageSize: Number(req.query["pageSize"] ?? "50"),
    });
    res.json(result);
  } catch (err) {
    handleGrpcError(res, err);
  }
});

// --- Error handling ---

function handleGrpcError(res: express.Response, err: unknown): void {
  const grpcErr = err as { code?: number; details?: string; message?: string };
  const statusMap: Record<number, number> = {
    0: 200,   // OK
    3: 400,   // INVALID_ARGUMENT
    5: 404,   // NOT_FOUND
    6: 409,   // ALREADY_EXISTS
    9: 400,   // FAILED_PRECONDITION
    13: 500,  // INTERNAL
    16: 401,  // UNAUTHENTICATED
  };
  const httpStatus = statusMap[grpcErr.code ?? 13] ?? 500;
  res.status(httpStatus).json({
    error: {
      code: grpcErr.code ?? 13,
      message: grpcErr.details ?? grpcErr.message ?? "Unknown error",
    },
  });
}

// --- Start ---

const BIND_HOST = process.env["BIND_HOST"] ?? "0.0.0.0";

clients = createClients(GRPC_SERVER);
app.listen(PORT, BIND_HOST, () => {
  console.log(`API proxy listening on http://${BIND_HOST}:${PORT}`);
  console.log(`Proxying gRPC calls to ${GRPC_SERVER}`);
});
