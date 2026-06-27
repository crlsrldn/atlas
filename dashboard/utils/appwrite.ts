import { Client, Account, Databases, ID } from "appwrite";

// Note: In a real app, these should come from Deno.env or public environment variables
// For this MVP, we will use placeholders or inject them at runtime.
// Normally Deno Fresh has plugins for env vars, but we can just use Deno.env in Deno contexts or window in browser contexts.

const endpoint = "https://cloud.appwrite.io/v1";
const projectId = "atlas"; // Default placeholder, ideally should be configurable

const client = new Client()
    .setEndpoint(endpoint)
    .setProject(projectId);

export const account = new Account(client);
export const databases = new Databases(client);
export { ID };
