import { useQuery } from "@tanstack/react-query";
import { Outlet } from "react-router";
import { get } from "../api/client";

export interface CurrentUser {
  pid: string;
  name: string;
  email: string;
}

/** The session is an HttpOnly cookie set by the server-rendered /login page,
 *  so ask the API who we are; a 401 makes the client redirect to /login. */
export function useCurrentUser() {
  return useQuery({ queryKey: ["auth", "current"], queryFn: () => get<CurrentUser>("/api/auth/current") });
}

export function RequireAuth() {
  const { isLoading, isError } = useCurrentUser();
  if (isLoading) return <p>Loading…</p>;
  if (isError) return null;
  return <Outlet />;
}
