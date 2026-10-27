import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
  type UseQueryResult,
} from "@tanstack/react-query";
import type { CreateCompany } from "../bindings/CreateCompany";
import type { Page } from "../bindings/Page";
import type { CompanyDto } from "../bindings/CompanyDto";
import type { UpdateCompany } from "../bindings/UpdateCompany";
import { ApiClientError, del, get, post, put } from "./client";

export interface ListCompaniesParams {
  page?: number;
  page_size?: number;
}

export const companyKeys = {
  all: ["companies"] as const,
  list: (params?: ListCompaniesParams) =>
    ["companies", "list", params ?? {}] as const,
  detail: (id: number) => ["companies", "detail", id] as const,
};

function buildQueryString(params?: ListCompaniesParams): string {
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

export function useListCompanies(
  params?: ListCompaniesParams,
): UseQueryResult<Page<CompanyDto>, ApiClientError> {
  return useQuery({
    queryKey: companyKeys.list(params),
    queryFn: () => get<Page<CompanyDto>>(`/api/companies${buildQueryString(params)}`),
  });
}

export function useCompany(id: number): UseQueryResult<CompanyDto, ApiClientError> {
  return useQuery({
    queryKey: companyKeys.detail(id),
    queryFn: () => get<CompanyDto>(`/api/companies/${id}`),
    enabled: Number.isFinite(id) && id > 0,
  });
}

export function useCreateCompany(): UseMutationResult<
  CompanyDto,
  ApiClientError,
  CreateCompany
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateCompany) => post<CompanyDto>("/api/companies", data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: companyKeys.all });
    },
  });
}

export interface UpdateCompanyVariables {
  id: number;
  data: UpdateCompany;
}

export function useUpdateCompany(): UseMutationResult<
  CompanyDto,
  ApiClientError,
  UpdateCompanyVariables
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: UpdateCompanyVariables) =>
      put<CompanyDto>(`/api/companies/${id}`, data),
    onSuccess: (_data, variables) => {
      void queryClient.invalidateQueries({ queryKey: companyKeys.all });
      void queryClient.invalidateQueries({
        queryKey: companyKeys.detail(variables.id),
      });
    },
  });
}

export function useRemoveCompany(): UseMutationResult<
  void,
  ApiClientError,
  number
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => del(`/api/companies/${id}`),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: companyKeys.all });
    },
  });
}
