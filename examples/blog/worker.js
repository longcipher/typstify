export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Handle static assets
    const page = await env.ASSETS.fetch(request);

    // Add security headers
    const response = new Response(page.body, page);
    response.headers.set('X-Frame-Options', 'DENY');
    response.headers.set('X-Content-Type-Options', 'nosniff');
    response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');

    return response;
  },
};