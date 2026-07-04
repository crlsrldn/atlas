import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals: { safeGetSession } }) => {
  return {
    session: (await safeGetSession()).session,
  };
};
