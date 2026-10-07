import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

export const errorRate = new Rate('errors');
export const shortenDuration = new Trend('shorten_duration');
export const redirectDuration = new Trend('redirect_duration');
export const healthDuration = new Trend('health_duration');

export const shortenRateLimited = new Counter('shorten_rate_limited');
export const redirectRateLimited = new Counter('redirect_rate_limited');
export const healthRateLimited = new Counter('health_rate_limited');

// .env values: 1000 req / 60s per endpoint ≈ 16.7 req/s
// Need >16.7 req/s from one IP to trigger limits
export const options = {
  stages: [
    { duration: '10s', target: 5 },     // warm up
    { duration: '30s', target: 100 },   // burst to exceed 1000/60s
    { duration: '10s', target: 0 },     // cool down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    errors: ['rate<0.5'], // rate limiting expected
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

function checkRateLimitHeaders(res, isLimited) {
  if (isLimited) {
    check(res, {
      '429 status': (r) => r.status === 429,
      'has Retry-After': (r) => r.headers['Retry-After'] !== undefined,
      'has X-RateLimit-Reset': (r) => r.headers['X-RateLimit-Reset'] !== undefined,
    });
  } else {
    check(res, {
      'has X-RateLimit-Limit': (r) => r.headers['X-RateLimit-Limit'] !== undefined,
      'has X-RateLimit-Reset': (r) => r.headers['X-RateLimit-Reset'] !== undefined,
    });
  }
}

export default function () {
  // === 1. Shorten ===
  const payload = JSON.stringify({
    url: `https://example.com/${Math.random().toString(36).slice(2)}`,
  });

  const shortenRes = http.post(`${BASE_URL}/api/shorten`, payload, {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'shorten' },
  });
  shortenDuration.add(shortenRes.timings.duration);

  const shortenLimited = shortenRes.status === 429;
  shortenRateLimited.add(shortenLimited ? 1 : 0);
  checkRateLimitHeaders(shortenRes, shortenLimited);

  if (shortenRes.status === 200) {
    check(shortenRes, { 'short URL returned': (r) => r.json('short_url') !== undefined });

    // === 2. Redirect (use the code we just created) ===
    const code = shortenRes.json('short_url')?.split('/').pop();
    if (code) {
      const redirectRes = http.get(`${BASE_URL}/${code}`, {
        redirects: 0,
        tags: { name: 'redirect' },
      });
      redirectDuration.add(redirectRes.timings.duration);

      const redirectLimited = redirectRes.status === 429;
      redirectRateLimited.add(redirectLimited ? 1 : 0);
      checkRateLimitHeaders(redirectRes, redirectLimited);

      if (!redirectLimited) {
        check(redirectRes, {
          'redirect 302': (r) => r.status === 302,
          'has Location': (r) => r.headers['Location'] !== undefined,
        });
      }
    }
  }

  // === 3. Health ===
  const healthRes = http.get(`${BASE_URL}/health`, { tags: { name: 'health' } });
  healthDuration.add(healthRes.timings.duration);

  const healthLimited = healthRes.status === 429;
  healthRateLimited.add(healthLimited ? 1 : 0);
  checkRateLimitHeaders(healthRes, healthLimited);

  if (!healthLimited) {
    check(healthRes, { 'health OK': (r) => r.status === 200 && r.body === 'OK' });
  }

  // Small sleep to control rate
  sleep(0.03);
}
