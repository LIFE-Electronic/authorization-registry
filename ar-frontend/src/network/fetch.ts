import { useAuth } from "../auth";
import { z } from "zod";

export const baseAPIUrl =
  import.meta.env.VITE_BASE_API_URL || window.location.origin.replace("/ui", "");

const errorResponseSchema = z.object({
  message: z.string(),
  error_type: z.object({
    type: z.string(),
    metadata: z.record(z.string()).optional(),
  }),
  errors: z
    .array(
      z.object({
        error_type: z.object({
          type: z.string(),
          metadata: z.record(z.string()).optional(),
        }),
        message: z.string(),
        location: z.string().optional(),
      }),
    )
    .optional(),
});

export class ErrorResponse extends Error {
  message: string;
  errors: Record<string, string>;
  response: Response;

  constructor(
    message: string,
    errors: Record<string, string>,
    response: Response,
  ) {
    super(message);

    this.message = message;
    this.errors = errors;
    this.response = response;
  }
}

async function createErrorResponse(response: Response): Promise<ErrorResponse> {
  const json = await response.json();

  const parsedResponse = errorResponseSchema.parse(json);
  const errors: Record<string, string> = {};
  parsedResponse.errors?.forEach((e, idx) => {
    errors[e.location || idx] = e.message;
  });

  const message = parsedResponse.message;

  return new ErrorResponse(message, errors, response);
}

export function useAuthenticatedFetch() {
  const { getToken } = useAuth();

  const authenticatedFetch: typeof fetch = async (
    input: RequestInfo | URL,
    init?: RequestInit,
  ) => {
    const token = await getToken();

    const headers = {
      Authorization: `Bearer ${token}`,
      ...init?.headers,
    };

    const response = await fetch(input, {
      ...init,
      headers,
    });

    if (!response.ok) {
      throw await createErrorResponse(response);
    }

    return response;
  };

  return authenticatedFetch;
}
