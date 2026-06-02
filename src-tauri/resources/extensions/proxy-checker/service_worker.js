const STORAGE_KEY = 'proxyCheckResult';
const PROXY_STATUS_KEY = 'yubaiProxyStatus';
const DEFAULT_PROXY_CONFIG = { enabled: false, mode: 'direct' };
let cachedProxyConfig = DEFAULT_PROXY_CONFIG;

function chromePromise(fn) {
  return new Promise((resolve, reject) => {
    fn((result) => {
      const error = chrome.runtime.lastError;
      if (error) {
        reject(new Error(error.message));
      } else {
        resolve(result);
      }
    });
  });
}

function proxyScheme(config) {
  return config.scheme === 'socks5' ? 'socks5'
    : config.scheme === 'socks4' ? 'socks4'
      : config.scheme === 'https' ? 'https'
        : 'http';
}

async function loadProxyConfig() {
  try {
    const response = await fetch(chrome.runtime.getURL('proxy_config.json'), {
      cache: 'no-store',
    });
    if (!response.ok) {
      throw new Error(`proxy_config.json HTTP ${response.status}`);
    }
    cachedProxyConfig = await response.json();
  } catch (error) {
    console.warn(error);
  }
  return cachedProxyConfig || DEFAULT_PROXY_CONFIG;
}

async function applyProxyConfig() {
  const proxyConfig = await loadProxyConfig();
  if (!proxyConfig.enabled) {
    await chrome.storage.local.set({
      [PROXY_STATUS_KEY]: {
        enabled: false,
        mode: 'direct',
        source: '',
        appliedAt: new Date().toISOString(),
      },
    });
    return;
  }

  const rules = {
    singleProxy: {
      scheme: proxyScheme(proxyConfig),
      host: proxyConfig.host,
      port: Number(proxyConfig.port),
    },
    bypassList: proxyConfig.bypassList || ['<local>'],
  };

  await chromePromise((done) => chrome.proxy.settings.set({
    value: {
      mode: 'fixed_servers',
      rules,
    },
    scope: 'regular',
  }, done));

  await chrome.storage.local.set({
    [PROXY_STATUS_KEY]: {
      enabled: true,
      mode: 'fixed_servers',
      scheme: proxyConfig.scheme,
      host: proxyConfig.host,
      port: proxyConfig.port,
      username: proxyConfig.username ? '***' : '',
      source: proxyConfig.source || '',
      appliedAt: new Date().toISOString(),
    },
  });
}

chrome.webRequest.onAuthRequired.addListener(
  (_details, callback) => {
    loadProxyConfig()
      .then((proxyConfig) => {
        if (!proxyConfig.enabled || !proxyConfig.username) {
          callback({});
          return;
        }
        callback({
          authCredentials: {
            username: proxyConfig.username,
            password: proxyConfig.password || '',
          },
        });
      })
      .catch(() => callback({}));
  },
  { urls: ['<all_urls>'] },
  ['asyncBlocking']
);

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type !== 'yubai:apply-proxy-config') {
    return false;
  }

  applyProxyConfig()
    .then(() => sendResponse({ ok: true }))
    .catch((error) => sendResponse({
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  return true;
});

async function readJson(url) {
  const response = await fetch(url, {
    cache: 'no-store',
    credentials: 'omit',
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return response.json();
}

async function readText(url) {
  const response = await fetch(url, {
    cache: 'no-store',
    credentials: 'omit',
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return response.text();
}

function normalizeIpApi(data) {
  if (!data || data.status !== 'success' || !data.query) {
    throw new Error('ip-api.com returned no IP');
  }

  return {
    ok: true,
    provider: 'ipApi',
    providerLabel: 'ip-api.com',
    ip: data.query,
    country: data.country || '',
    countryCode: data.countryCode || '',
    region: data.regionName || '',
    city: data.city || '',
    timezone: data.timezone || '',
    isp: data.isp || '',
    asn: data.as || '',
    org: data.org || '',
    checkedAt: new Date().toISOString(),
  };
}

function normalizeCloudflareTrace(text) {
  const entries = Object.fromEntries(
    (text || '')
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const index = line.indexOf('=');
        return index === -1 ? [line, ''] : [line.slice(0, index), line.slice(index + 1)];
      })
  );

  if (!entries.ip) {
    throw new Error('cloudflare trace returned no IP');
  }

  return {
    ok: true,
    provider: 'cloudflare',
    providerLabel: 'Cloudflare',
    ip: entries.ip,
    country: entries.loc || '',
    colo: entries.colo || '',
    http: entries.http || '',
    tls: entries.tls || '',
    warp: entries.warp || '',
    checkedAt: new Date().toISOString(),
    raw: entries,
  };
}

function failedResult(provider, providerLabel, error) {
  return {
    ok: false,
    provider,
    providerLabel,
    ip: '',
    checkedAt: new Date().toISOString(),
    error: error instanceof Error ? error.message : String(error),
  };
}

async function detectEgress() {
  try {
    const ipApi = normalizeIpApi(await readJson('http://ip-api.com/json/?lang=zh-CN'));
    return {
      ok: true,
      activeProvider: 'ipApi',
      checkedAt: new Date().toISOString(),
      results: { ipApi },
    };
  } catch (error) {
    const ipApi = failedResult('ipApi', 'ip-api.com', error);
    try {
      const cloudflare = normalizeCloudflareTrace(await readText('https://cloudflare.com/cdn-cgi/trace'));
      return {
        ok: true,
        activeProvider: 'cloudflare',
        checkedAt: new Date().toISOString(),
        results: { ipApi, cloudflare },
      };
    } catch (cloudflareError) {
      return {
        ok: false,
        activeProvider: '',
        checkedAt: new Date().toISOString(),
        results: {
          ipApi,
          cloudflare: failedResult('cloudflare', 'Cloudflare', cloudflareError),
        },
      };
    }
  }
}

async function saveResult(result) {
  await chrome.storage.local.set({ [STORAGE_KEY]: result });
  return result;
}

chrome.runtime.onInstalled.addListener(() => {
  applyProxyConfig().catch(console.warn);
  detectEgress().then(saveResult).catch(console.warn);
});

chrome.runtime.onStartup.addListener(() => {
  applyProxyConfig().catch(console.warn);
  detectEgress().then(saveResult).catch(console.warn);
});

applyProxyConfig().catch(console.warn);
