import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

interface AuthState {
  userId: string | null;
}

const STORAGE_KEY = "it_user_id";

function loadUserId(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

const initialState: AuthState = {
  userId: loadUserId(),
};

const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    login(state, action: PayloadAction<string>) {
      state.userId = action.payload;
      localStorage.setItem(STORAGE_KEY, action.payload);
    },
    logout(state) {
      state.userId = null;
      localStorage.removeItem(STORAGE_KEY);
    },
  },
});

export const { login, logout } = authSlice.actions;
export default authSlice.reducer;
