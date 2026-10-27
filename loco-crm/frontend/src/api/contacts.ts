import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
  type UseQueryResult,
} from "@tanstack/react-query";
import type { CreateContact } from "../bindings/CreateContact";
import type { Page } from "../bindings/Page";
import type { ContactDto } from "../bindings/ContactDto";
import type { UpdateContact } from "../bindings/UpdateContact";
import { ApiClientError, del, get, post, put } from "./client";

export interface ListContactsParams {
  page?: number;
  page_size?: number;
}

export const contactKeys = {
  all: ["contacts"] as const,
  list: (params?: ListContactsParams) =>
    ["contacts", "list", params ?? {}] as const,
  detail: (id: number) => ["contacts", "detail", id] as const,
};

function buildQueryString(params?: ListContactsParams): string {
  if (!params) {
    return "";
  }
  const search = new URLSearchParams();
  if (params.page !== undefined) {
    search.set("page", String(params.page));
  }
  if (params.page_size !== undefined) {
    search.set("page_size", String(params.page_size));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

export function useListContacts(
  params?: ListContactsParams,
): UseQueryResult<Page<ContactDto>, ApiClientError> {
  return useQuery({
    queryKey: contactKeys.list(params),
    queryFn: () => get<Page<ContactDto>>(`/api/contacts${buildQueryString(params)}`),
  });
}

export function useContact(id: number): UseQueryResult<ContactDto, ApiClientError> {
  return useQuery({
    queryKey: contactKeys.detail(id),
    queryFn: () => get<ContactDto>(`/api/contacts/${id}`),
    enabled: Number.isFinite(id) && id > 0,
  });
}

export function useCreateContact(): UseMutationResult<
  ContactDto,
  ApiClientError,
  CreateContact
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateContact) => post<ContactDto>("/api/contacts", data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: contactKeys.all });
    },
  });
}

export interface UpdateContactVariables {
  id: number;
  data: UpdateContact;
}

export function useUpdateContact(): UseMutationResult<
  ContactDto,
  ApiClientError,
  UpdateContactVariables
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: UpdateContactVariables) =>
      put<ContactDto>(`/api/contacts/${id}`, data),
    onSuccess: (_data, variables) => {
      void queryClient.invalidateQueries({ queryKey: contactKeys.all });
      void queryClient.invalidateQueries({
        queryKey: contactKeys.detail(variables.id),
      });
    },
  });
}

export function useRemoveContact(): UseMutationResult<
  void,
  ApiClientError,
  number
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => del(`/api/contacts/${id}`),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: contactKeys.all });
    },
  });
}
