const STORAGE_KEY = 'proxyCheckResult';
const PROXY_STATUS_KEY = 'yubaiProxyStatus';

const elements = {
  statusBadge: document.getElementById('statusBadge'),
  proxyStatusText: document.getElementById('proxyStatusText'),
  proxyDetailText: document.getElementById('proxyDetailText'),
  providerIndex: document.getElementById('providerIndex'),
  providerTitle: document.getElementById('providerTitle'),
  providerStatus: document.getElementById('providerStatus'),
  resultFields: document.getElementById('resultFields'),
  providerErrorText: document.getElementById('providerErrorText'),
  timeValue: document.getElementById('timeValue'),
  errorText: document.getElementById('errorText'),
  refreshButton: document.getElementById('refreshButton'),
  cloudflareButton: document.getElementById('cloudflareButton'),
};

function fallback(value) {
  return value && String(value).trim() ? value : '-';
}

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
    throw new Error(`ip-api.com 返回异常: ${JSON.stringify(data || {})}`);
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
    throw new Error('Cloudflare trace 未返回 ip 字段');
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

async function detectWithIpApiFallback() {
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

async function detectCloudflareOnly(existingResult) {
  try {
    const cloudflare = normalizeCloudflareTrace(await readText('https://cloudflare.com/cdn-cgi/trace'));
    return {
      ok: true,
      activeProvider: 'cloudflare',
      checkedAt: new Date().toISOString(),
      results: {
        ...(existingResult?.results || {}),
        cloudflare,
      },
    };
  } catch (error) {
    return {
      ok: false,
      activeProvider: 'cloudflare',
      checkedAt: new Date().toISOString(),
      results: {
        ...(existingResult?.results || {}),
        cloudflare: failedResult('cloudflare', 'Cloudflare', error),
      },
    };
  }
}

function formatTime(value) {
  if (!value) {
    return '-';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString('zh-CN');
}

function setStatus(kind, label) {
  elements.statusBadge.className = `badge badge--${kind}`;
  elements.statusBadge.textContent = label;
}

function renderProxyStatus(status) {
  if (!status) {
    elements.proxyStatusText.textContent = '未启用';
    elements.proxyDetailText.textContent = '当前浏览器未通过插件设置代理';
    return;
  }

  if (!status.enabled) {
    elements.proxyStatusText.textContent = '直连';
    elements.proxyDetailText.textContent = status.appliedAt ? `应用时间：${formatTime(status.appliedAt)}` : '未设置代理';
    return;
  }

  elements.proxyStatusText.textContent = `${status.scheme || '-'}://${status.host || '-'}:${status.port || '-'}`;
  elements.proxyDetailText.textContent = [
    status.username ? '认证：已配置' : '认证：无',
    status.source ? `来源：${status.source}` : '',
    status.appliedAt ? `应用时间：${formatTime(status.appliedAt)}` : '',
  ].filter(Boolean).join(' / ');
}

function setProviderStatus(result) {
  const kind = result && result.ok ? 'ok' : result ? 'failed' : 'checking';
  const label = result && result.ok ? '成功' : result ? '失败' : '检测中';
  elements.providerStatus.className = `provider-status provider-status--${kind}`;
  elements.providerStatus.textContent = label;
}

function field(label, value, wide = false) {
  const item = document.createElement('div');
  item.className = wide ? 'field field--wide' : 'field';

  const labelNode = document.createElement('span');
  labelNode.textContent = label;

  const valueNode = document.createElement('strong');
  valueNode.textContent = fallback(value);

  item.append(labelNode, valueNode);
  return item;
}

function renderIpApi(result) {
  elements.providerIndex.textContent = '01';
  elements.providerTitle.textContent = 'ip-api.com';
  setProviderStatus(result);
  elements.resultFields.replaceChildren(
    field('出口 IP', result?.ip, true),
    field('位置', [result?.country, result?.region, result?.city].filter(Boolean).join(' / '), true),
    field('ISP / 组织', [result?.isp, result?.org].filter(Boolean).join(' / '), true),
    field('ASN', result?.asn),
    field('时区', result?.timezone)
  );
}

function renderCloudflare(result) {
  elements.providerIndex.textContent = '02';
  elements.providerTitle.textContent = 'Cloudflare';
  setProviderStatus(result);
  elements.resultFields.replaceChildren(
    field('出口 IP', result?.ip, true),
    field('国家/地区', result?.country),
    field('节点', result?.colo),
    field('HTTP', result?.http),
    field('TLS', result?.tls),
    field('WARP', result?.warp, true)
  );
}

function renderProviderError(result) {
  if (result && result.error) {
    elements.providerErrorText.hidden = false;
    elements.providerErrorText.textContent = result.error;
  } else {
    elements.providerErrorText.hidden = true;
    elements.providerErrorText.textContent = '';
  }
}

function activeResult(result) {
  if (result?.activeProvider === 'cloudflare') {
    return result.results?.cloudflare;
  }
  if (result?.activeProvider === 'ipApi') {
    return result.results?.ipApi;
  }
  return result?.results?.ipApi || result?.results?.cloudflare || null;
}

function render(result) {
  if (!result) {
    setStatus('checking', '未检测');
    elements.providerIndex.textContent = '01';
    elements.providerTitle.textContent = 'ip-api.com';
    setProviderStatus(null);
    elements.resultFields.replaceChildren(
      field('出口 IP', '-', true),
      field('位置', '-', true),
      field('ISP / 组织', '-', true),
      field('ASN', '-'),
      field('时区', '-')
    );
    elements.cloudflareButton.hidden = true;
    return;
  }

  const current = activeResult(result);
  setStatus(result.ok ? 'ok' : 'failed', result.ok ? '已检测' : '检测失败');

  if (result.activeProvider === 'cloudflare') {
    renderCloudflare(current);
  } else {
    renderIpApi(current);
  }

  renderProviderError(current);
  elements.timeValue.textContent = formatTime(result.checkedAt);
  elements.cloudflareButton.hidden = !(result.activeProvider === 'ipApi' && result.results?.ipApi?.ok);

  const ipApiError = result.results?.ipApi?.error;
  const cloudflareError = result.results?.cloudflare?.error;
  if (!result.ok && (ipApiError || cloudflareError)) {
    elements.errorText.hidden = false;
    elements.errorText.textContent = [ipApiError, cloudflareError].filter(Boolean).join('；');
  } else {
    elements.errorText.hidden = true;
    elements.errorText.textContent = '';
  }
}

async function saveAndRender(result) {
  await chrome.storage.local.set({ [STORAGE_KEY]: result });
  render(result);
  return result;
}

async function loadCachedResult() {
  await chrome.runtime.sendMessage({ type: 'yubai:apply-proxy-config' }).catch(() => null);
  const stored = await chrome.storage.local.get([STORAGE_KEY, PROXY_STATUS_KEY]);
  renderProxyStatus(stored[PROXY_STATUS_KEY]);
  render(stored[STORAGE_KEY]);
  return stored[STORAGE_KEY];
}

async function refresh() {
  elements.refreshButton.disabled = true;
  elements.cloudflareButton.disabled = true;
  setStatus('checking', '检测中');
  try {
    await chrome.runtime.sendMessage({ type: 'yubai:apply-proxy-config' }).catch(() => null);
    const stored = await chrome.storage.local.get(PROXY_STATUS_KEY);
    renderProxyStatus(stored[PROXY_STATUS_KEY]);
    await saveAndRender(await detectWithIpApiFallback());
  } finally {
    elements.refreshButton.disabled = false;
    elements.cloudflareButton.disabled = false;
  }
}

async function checkCloudflare() {
  elements.refreshButton.disabled = true;
  elements.cloudflareButton.disabled = true;
  setStatus('checking', '检测中');
  try {
    const stored = await chrome.storage.local.get(STORAGE_KEY);
    await saveAndRender(await detectCloudflareOnly(stored[STORAGE_KEY]));
  } finally {
    elements.refreshButton.disabled = false;
    elements.cloudflareButton.disabled = false;
  }
}

elements.refreshButton.addEventListener('click', refresh);
elements.cloudflareButton.addEventListener('click', checkCloudflare);

loadCachedResult().finally(refresh);
