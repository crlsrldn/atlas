import { createBrowserClient, isBrowser } from '@supabase/ssr';
import type { LayoutLoad } from './$types';
import { env } from '$env/dynamic/public';

export const load: LayoutLoad = async ({ fetch, data, depends }) => {
  depends('supabase:auth');

  const supabase = isBrowser()
    ? createBrowserClient(
        env.PUBLIC_SUPABASE_URL || 'https://dummy.supabase.co',
        env.PUBLIC_SUPABASE_ANON_KEY || 'dummy',
        {
          global: {
            fetch,
          },
        }
      )
    : (data as any).supabase;

  return { supabase, session: data.session };
};
