import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, fetch }) => {
  const response = await fetch(`/api/snippets/${params.id}`);

  if (!response.ok) {
    throw redirect(302, "/");
  }

  const snippet = await response.json();

  return {
    snippet,
  };
};
