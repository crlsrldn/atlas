import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const supabaseUrl = env.SUPABASE_URL;
	const serviceKey = env.SUPABASE_SERVICE_ROLE_KEY;

	if (!supabaseUrl || !serviceKey) {
		// Silently succeed if telemetry is not configured
		return json({ success: true });
	}

	try {
		const payload = await request.json();
		const { event_type, event_data } = payload;

		if (!event_type || !event_data) {
			return json({ error: 'Missing event_type or event_data' }, { status: 400 });
		}

		const response = await fetch(`${supabaseUrl}/rest/v1/telemetry`, {
			method: 'POST',
			headers: {
				apikey: serviceKey,
				Authorization: `Bearer ${serviceKey}`,
				'Content-Type': 'application/json',
				Prefer: 'return=minimal'
			},
			body: JSON.stringify({
				event_type,
				event_data
			})
		});

		if (!response.ok) {
			console.error('Failed to log telemetry from dashboard:', await response.text());
		}
	} catch (e) {
		console.error('Telemetry error:', e);
	}

	return json({ success: true });
};
