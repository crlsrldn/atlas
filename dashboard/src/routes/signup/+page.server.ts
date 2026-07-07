import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';
import { createClient } from '@supabase/supabase-js';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async () => {
  const supabase = createClient(publicEnv.PUBLIC_SUPABASE_URL as string, env.SUPABASE_SERVICE_ROLE_KEY as string);
  
  const { data: settings } = await supabase.from('app_settings').select('*').eq('key', 'signups_open').single();
  const signupsOpen = settings?.value ?? false;

  return {
    signupsOpen
  };
};

export const actions: Actions = {
  waitlist: async ({ request }) => {
    const supabase = createClient(publicEnv.PUBLIC_SUPABASE_URL as string, env.SUPABASE_SERVICE_ROLE_KEY as string);
    const form = await request.formData();
    const email = form.get('email')?.toString().trim();

    if (!email) return fail(400, { error: 'Email is required' });

    const { error } = await supabase.from('waitlist').insert({ email });
    if (error) {
      if (error.code === '23505') {
        return { success: true, message: 'You are already on the waitlist!' };
      }
      return fail(500, { error: 'Failed to join waitlist' });
    }

    return { success: true, message: 'You have been added to the waitlist!' };
  },

  signupWithInvite: async ({ request }) => {
    const supabase = createClient(publicEnv.PUBLIC_SUPABASE_URL as string, env.SUPABASE_SERVICE_ROLE_KEY as string);
    const form = await request.formData();
    const email = form.get('email')?.toString().trim();
    const password = form.get('password')?.toString();
    const code = form.get('code')?.toString().trim();

    if (!email || !password || !code) {
      return fail(400, { error: 'Email, password, and invite code are required' });
    }

    // 1. Validate invite code
    const { data: invite } = await supabase.from('invites').select('*').eq('code', code).is('used_by', null).single();
    if (!invite) {
      return fail(400, { error: 'Invalid or already used invite code' });
    }

    // 2. Create user via Supabase Auth Admin
    const { data: authData, error: authError } = await supabase.auth.admin.createUser({
      email,
      password,
      email_confirm: true // Auto-confirm for beta testers
    });

    if (authError || !authData.user) {
      return fail(400, { error: authError?.message || 'Failed to create user' });
    }

    const userId = authData.user.id;

    // 3. Mark invite as used
    await supabase.from('invites').update({ used_by: userId, used_at: new Date().toISOString() }).eq('id', invite.id);

    // 4. Create app_users row for premium tier
    await supabase.from('app_users').insert({
      id: userId,
      tier: invite.tier_granted || 'premium'
    });

    // 5. Generate 1 referral invite for the new user
    const referralCode = 'ATLAS-' + Math.random().toString(36).substring(2, 8).toUpperCase();
    await supabase.from('invites').insert({
      code: referralCode,
      created_by: userId,
      tier_granted: 'premium'
    });

    // Since we used admin.createUser, the user is NOT logged in yet. 
    // They will need to log in normally using the login page.
    throw redirect(303, '/login?success=Account created! Please sign in.');
  }
};
