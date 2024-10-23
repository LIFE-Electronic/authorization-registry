import { useAuth } from "../auth";
import { z } from "zod";

export const baseAPIUrl =
  import.meta.env.VITE_BASE_API_URL ||
  window.location.origin.replace("/ui", "");

const errorResponseSchema = z.object({
  error: z.string(),
});

export class ErrorResponse extends Error {
  message: string;
  response: Response;

  constructor(message: string, response: Response) {
    super(message);

    this.message = message;
    this.response = response;
  }
}

async function createErrorResponse(response: Response): Promise<ErrorResponse> {
  const json = await response.json();

  const parsedResponse = errorResponseSchema.parse(json);

  const message = parsedResponse.error;

  return new ErrorResponse(message, response);
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
