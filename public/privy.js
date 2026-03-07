import Privy, { LocalStorage } from "@privy-io/js-sdk-core";

// ---------------------------------------------------------------------------
// Module-level state
// ---------------------------------------------------------------------------
/** @type {InstanceType<typeof Privy> | null} */
let privy = null;
let _appId = "";
let _session = null; // { token, refresh_token, user }

const PRIVY_AUTH_URL = "https://auth.privy.io/api/v1";
const SESSION_KEY = "privy_bridge_session";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function assertReady() {
    if (!_appId) throw new Error("Bridge not initialised — call init() first");
}

/** Common headers required on every Privy auth REST call */
function authHeaders() {
    return {
        "Content-Type": "application/json",
        "privy-app-id": _appId,
    };
}

/**
 * POST to a Privy auth endpoint, throw on non-2xx.
 * @param {string} path  — e.g. "/siwe/init"
 * @param {object} body
 */
async function privyPost(path, body) {
    const res = await fetch(`${PRIVY_AUTH_URL}${path}`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify(body),
    });

    const json = await res.json().catch(() => ({}));

    if (!res.ok) {
        const msg = json?.message ?? json?.error ?? `HTTP ${res.status}`;
        throw new Error(`Privy API error (${path}): ${msg}`);
    }

    return json;
}

/** Normalise the user object returned by either REST or sdk-core */
function serializeUser(user) {
    if (!user) return null;

    // REST API returns linked_accounts (snake_case); sdk-core returns linkedAccounts
    const linked = user.linked_accounts ?? user.linkedAccounts ?? [];

    const wallet =
        linked.find((a) => a.type === "wallet" && a.chain_type === "ethereum") ??
        linked.find((a) => a.type === "wallet" && a.chain_type === "solana") ??
        null;

    return {
        id: user.id,
        linkedAccounts: linked,
        wallet: wallet
            ? {
                address: wallet.address,
                chainType: wallet.chain_type,
                walletClient: wallet.wallet_client_type ?? wallet.connector_type ?? null,
            }
            : null,
        createdAt: user.created_at ?? user.createdAt ?? null,
    };
}

/** Persist session to localStorage so page reloads restore auth state */
function saveSession(session) {
    _session = session;
    try { localStorage.setItem(SESSION_KEY, JSON.stringify(session)); } catch (_) { }
}

function clearSession() {
    _session = null;
    try { localStorage.removeItem(SESSION_KEY); } catch (_) { }
}

function loadSession() {
    try {
        const raw = localStorage.getItem(SESSION_KEY);
        if (raw) _session = JSON.parse(raw);
    } catch (_) { }
}

// ---------------------------------------------------------------------------
// Named exports — wasm-bindgen binds these via module = "/privy-bridge.js"
// ---------------------------------------------------------------------------

/**
 * Initialise the bridge. Must be called (and awaited) before any login.
 */
export async function init(appId, clientId) {
    _appId = appId;
    privy = new Privy({ appId, clientId, storage: new LocalStorage() });
    loadSession();
}

// ---------------------------------------------------------------------------
// Wallet connect — called directly from JS to preserve the browser's
// user-gesture context. Rust's spawn_local breaks the gesture chain,
// causing wallet popups to be silently blocked.
// ---------------------------------------------------------------------------

/**
 * Trigger MetaMask connect popup and return the checksummed address.
 * Must be called from within a click handler (user gesture).
 * @returns {Promise<string>} checksummed Ethereum address
 */
export async function connectMetaMask() {
    const ethereum = window.ethereum;
    if (!ethereum?.isMetaMask) {
        throw new Error("MetaMask not found — install the extension");
    }
    // eth_requestAccounts triggers the popup; its return value is the
    // authorised account list (checksummed). Do NOT call eth_accounts
    // separately — that returns [] silently if not yet authorised.
    const accounts = await ethereum.request({ method: "eth_requestAccounts" });
    if (!accounts?.length) throw new Error("No account returned from MetaMask");
    return accounts[0];
}

/**
 * Trigger Phantom connect popup and return the base58 public key.
 * Must be called from within a click handler (user gesture).
 * @returns {Promise<string>} Solana public key (base58)
 */
export async function connectPhantom() {
    const solana = window.solana;
    if (!solana?.isPhantom) {
        throw new Error("Phantom not found — install the extension");
    }
    await solana.connect();
    const address = solana.publicKey?.toString();
    if (!address) throw new Error("No public key returned from Phantom");
    return address;
}

/**
 * Disconnect Phantom. MetaMask has no disconnect API — clear state in Rust.
 */
export async function disconnectPhantom() {
    const solana = window.solana;
    if (solana?.isPhantom) {
        try { await solana.disconnect(); } catch (_) { }
    }
}


// ---------------------------------------------------------------------------

// flow_context is returned by /passwordless/init and must be echoed back
// to /passwordless/authenticate to tie the OTP to the correct session.
let _emailFlowContext = null;

/**
 * Step 1 — send a 6-digit OTP to the user's email.
 * Privy endpoint: POST /passwordless/init
 */
export async function sendEmailCode(email) {
    assertReady();
    const res = await privyPost("/passwordless/init", {
        email,
        locale: "en",
    });
    // Persist the flow_context so verifyEmailCode can echo it back.
    _emailFlowContext = res?.flow_context ?? null;
}

/**
 * Step 2 — verify the OTP and exchange it for a session.
 * Privy endpoint: POST /passwordless/authenticate
 * @returns serialised user object
 */
export async function verifyEmailCode(email, code) {
    assertReady();

    const body = {
        email,
        code: code.trim(),
        mode: "login-or-sign-up",
    };

    // flow_context links this request back to the init call — required by Privy.
    if (_emailFlowContext) body.flow_context = _emailFlowContext;

    const session = await privyPost("/passwordless/authenticate", body);

    _emailFlowContext = null; // consumed — clear for next login attempt
    saveSession(session);
    return serializeUser(session.user);
}


export async function logout() {
    assertReady();
    clearSession();
    try { await privy?.auth?.logout?.(); } catch (_) { }
}

export function getUser() {
    if (_session?.user) return serializeUser(_session.user);
    if (privy?.user) return serializeUser(privy.user);
    return null;
}

export function isAuthenticated() {
    return !!(_session?.token ?? privy?.user);
}

export function getAccessToken() {
    return _session?.token ?? null;
}