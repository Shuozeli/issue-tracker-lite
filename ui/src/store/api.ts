import { createApi, fetchBaseQuery } from "@reduxjs/toolkit/query/react";
import type {
  Component,
  Issue,
  Comment,
  CommentRevision,
  Hotlist,
  HotlistIssue,
  Event,
  CreateComponentRequest,
  CreateIssueRequest,
  UpdateIssueRequest,
  CreateCommentRequest,
  UpdateCommentRequest,
  CreateHotlistRequest,
} from "../api/types";
interface AuthState {
  userId: string | null;
}

interface ListComponentsResponse {
  components: Component[];
  nextPageToken: string;
}

interface ListCommentsResponse {
  comments: Comment[];
  nextPageToken: string;
}

interface ListHotlistsResponse {
  hotlists: Hotlist[];
  nextPageToken: string;
}

interface ListHotlistIssuesResponse {
  issues: HotlistIssue[];
}

interface ListEventsResponse {
  events: Event[];
  nextPageToken: string;
}

interface SearchIssuesResponse {
  issues: Issue[];
  totalCount: number;
  nextPageToken: string;
}

interface ListCommentRevisionsResponse {
  revisions: CommentRevision[];
  nextPageToken: string;
}

export const api = createApi({
  reducerPath: "api",
  baseQuery: fetchBaseQuery({
    baseUrl: "/api",
    prepareHeaders: (headers, { getState }) => {
      const state = getState() as { auth?: AuthState };
      if (state.auth?.userId) {
        headers.set("x-user-id", state.auth.userId);
      }
      return headers;
    },
  }),
  tagTypes: ["Component", "Issue", "Comment", "Hotlist", "Event"],
  endpoints: (builder) => ({
    // Components
    listComponents: builder.query<ListComponentsResponse, void>({
      query: () => "/components",
      providesTags: ["Component"],
    }),
    getComponent: builder.query<Component, number>({
      query: (id) => `/components/${id}`,
      providesTags: (_r, _e, id) => [{ type: "Component", id }],
    }),
    createComponent: builder.mutation<Component, CreateComponentRequest>({
      query: (body) => ({ url: "/components", method: "POST", body }),
      invalidatesTags: ["Component"],
    }),
    updateComponent: builder.mutation<Component, { id: number; name?: string; description?: string }>({
      query: ({ id, ...body }) => ({ url: `/components/${id}`, method: "PATCH", body }),
      invalidatesTags: ["Component"],
    }),
    deleteComponent: builder.mutation<unknown, number>({
      query: (id) => ({ url: `/components/${id}`, method: "DELETE" }),
      invalidatesTags: ["Component"],
    }),

    // Issues
    listIssues: builder.query<SearchIssuesResponse, { componentId?: number } | void>({
      query: (params) => {
        if (params?.componentId) {
          return `/issues?componentId=${params.componentId}`;
        }
        return "/search?q=";
      },
      providesTags: ["Issue"],
    }),
    getIssue: builder.query<Issue, number>({
      query: (id) => `/issues/${id}`,
      providesTags: (_r, _e, id) => [{ type: "Issue", id }],
    }),
    createIssue: builder.mutation<Issue, CreateIssueRequest>({
      query: (body) => ({ url: "/issues", method: "POST", body }),
      invalidatesTags: ["Issue", "Component"],
    }),
    updateIssue: builder.mutation<Issue, { id: number } & UpdateIssueRequest>({
      query: ({ id, ...body }) => ({ url: `/issues/${id}`, method: "PATCH", body }),
      invalidatesTags: ["Issue"],
    }),

    // Comments
    listComments: builder.query<ListCommentsResponse, number>({
      query: (issueId) => `/issues/${issueId}/comments`,
      providesTags: (_r, _e, issueId) => [{ type: "Comment", id: issueId }],
    }),
    createComment: builder.mutation<Comment, { issueId: number } & CreateCommentRequest>({
      query: ({ issueId, ...body }) => ({
        url: `/issues/${issueId}/comments`,
        method: "POST",
        body,
      }),
      invalidatesTags: (_r, _e, { issueId }) => [{ type: "Comment", id: issueId }],
    }),
    updateComment: builder.mutation<Comment, UpdateCommentRequest>({
      query: ({ commentId, body }) => ({
        url: `/comments/${commentId}`,
        method: "PATCH",
        body: { body },
      }),
      invalidatesTags: ["Comment"],
    }),
    hideComment: builder.mutation<Comment, { commentId: number }>({
      query: ({ commentId }) => ({
        url: `/comments/${commentId}/hide`,
        method: "POST",
        body: { hidden: true },
      }),
      invalidatesTags: ["Comment"],
    }),
    listCommentRevisions: builder.query<ListCommentRevisionsResponse, number>({
      query: (commentId) => `/comments/${commentId}/revisions`,
    }),

    // Hotlists
    listHotlists: builder.query<ListHotlistsResponse, void>({
      query: () => "/hotlists",
      providesTags: ["Hotlist"],
    }),
    getHotlist: builder.query<Hotlist, number>({
      query: (id) => `/hotlists/${id}`,
      providesTags: (_r, _e, id) => [{ type: "Hotlist", id }],
    }),
    createHotlist: builder.mutation<Hotlist, CreateHotlistRequest>({
      query: (body) => ({ url: "/hotlists", method: "POST", body }),
      invalidatesTags: ["Hotlist"],
    }),
    listHotlistIssues: builder.query<ListHotlistIssuesResponse, number>({
      query: (hotlistId) => `/hotlists/${hotlistId}/issues`,
    }),

    // Search
    searchIssues: builder.query<SearchIssuesResponse, string>({
      query: (q) => `/search?q=${encodeURIComponent(q)}`,
    }),

    // Events
    listEvents: builder.query<ListEventsResponse, { entityType?: string; entityId?: number } | void>({
      query: (params) => {
        const q = new URLSearchParams();
        if (params?.entityType) q.set("entityType", params.entityType);
        if (params?.entityId) q.set("entityId", String(params.entityId));
        const qs = q.toString();
        return `/events${qs ? `?${qs}` : ""}`;
      },
      providesTags: ["Event"],
    }),
  }),
});

export const {
  useListComponentsQuery,
  useGetComponentQuery,
  useCreateComponentMutation,
  useUpdateComponentMutation,
  useDeleteComponentMutation,
  useListIssuesQuery,
  useGetIssueQuery,
  useCreateIssueMutation,
  useUpdateIssueMutation,
  useListCommentsQuery,
  useCreateCommentMutation,
  useUpdateCommentMutation,
  useHideCommentMutation,
  useListCommentRevisionsQuery,
  useListHotlistsQuery,
  useGetHotlistQuery,
  useCreateHotlistMutation,
  useListHotlistIssuesQuery,
  useSearchIssuesQuery,
  useListEventsQuery,
} = api;
