import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
  type UseQueryResult,
} from "@tanstack/react-query";
import type { CreateDeal } from "../bindings/CreateDeal";
import type { Page } from "../bindings/Page";
import type { DealDto } from "../bindings/DealDto";
import type { UpdateDeal } from "../bindings/UpdateDeal";
import { ApiClientError, del, get, post, put } from "./client";

export interface ListDealsParams {
  page?: number;
  page_size?: number;
}

export const dealKeys = {
  all: ["deals"] as const,
  list: (params?: ListDealsParams) =>
    ["deals", "list", params ?? {}] as const,
  detail: (id: number) => ["deals", "detail", id] as const,
};

function buildQueryString(params?: ListDealsParams): string {
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

export function useListDeals(
  params?: ListDealsParams,
): UseQueryResult<Page<DealDto>, ApiClientError> {
  return useQuery({
    queryKey: dealKeys.list(params),
    queryFn: () => get<Page<DealDto>>(`/api/deals${buildQueryString(params)}`),
  });
}

export function useDeal(id: number): UseQueryResult<DealDto, ApiClientError> {
  return useQuery({
    queryKey: dealKeys.detail(id),
    queryFn: () => get<DealDto>(`/api/deals/${id}`),
    enabled: Number.isFinite(id) && id > 0,
  });
}

export function useCreateDeal(): UseMutationResult<
  DealDto,
  ApiClientError,
  CreateDeal
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateDeal) => post<DealDto>("/api/deals", data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: dealKeys.all });
    },
  });
}

export interface UpdateDealVariables {
  id: number;
  data: UpdateDeal;
}

export function useUpdateDeal(): UseMutationResult<
  DealDto,
  ApiClientError,
  UpdateDealVariables
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: UpdateDealVariables) =>
      put<DealDto>(`/api/deals/${id}`, data),
    onSuccess: (_data, variables) => {
      void queryClient.invalidateQueries({ queryKey: dealKeys.all });
      void queryClient.invalidateQueries({
        queryKey: dealKeys.detail(variables.id),
      });
    },
  });
}

export function useRemoveDeal(): UseMutationResult<
  void,
  ApiClientError,
  number
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => del(`/api/deals/${id}`),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: dealKeys.all });
    },
  });
}
