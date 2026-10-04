import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

export const errorRate = new Rate('errors');
export const shortenDuration = new Trend('shorten_duration');
export const redirectDuration = new Trend('redirect_duration');

export const options = {
  stages: [
    { duration: '30s', target: 50 },   // ramp up
    { duration: '1m', target: 100 },   // steady load
    { duration: '30s', target: 200 },  // peak
    { duration: '30s', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    errors: ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const codes = new Array(1000).fill(0).map((_, i) => `test${i}`);

export default function () {
  // 1. Create short URL
  const payload = JSON.stringify({
    url: `https://example.com/path/${Math.random().toString(36).slice(2)}`,
  });

  const params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'shorten' },
  };

  const shortenRes = http.post(`${BASE_URL}/api/shorten`, payload, params);
  shortenDuration.add(shortenRes.timings.duration);

  check(shortenRes, {
    'shorten status 200': (r) => r.status === 200,
    'shorten has short_url': (r) => r.json('short_url') !== undefined,
  }) || errorRate.add(1);

  const shortUrl = shortenRes.json('short_url');
  const code = shortUrl?.split('/').pop();

  // 2. Redirect (immediate)
  if (code) {
    const redirectRes = http.get(`${BASE_URL}/${code}`, {
      redirects: 0,
      tags: { name: 'redirect' },
    });
    redirectDuration.add(redirectRes.timings.duration);

    check(redirectRes, {
      'redirect status 302': (r) => r.status === 302,
      'redirect has location': (r) => r.headers.Location !== undefined,
    }) || errorRate.add(1);
  }

  // 3. Health check (occasional)
  if (Math.random() < 0.1) {
    const healthRes = http.get(`${BASE_URL}/health`, { tags: { name: 'health' } });
    check(healthRes, { 'health OK': (r) => r.status === 200 && r.body === 'OK' }) || errorRate.add(1);
  }

  sleep(Math.random() * 0.5 + 0.1); // 100-600ms think time
}