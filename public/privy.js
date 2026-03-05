/**
 * privy-bridge.js
 *
 * Framework-agnostic Privy bridge for WASM interop.
 * Uses @privy-io/js-sdk-core — install with:
 *   npm install @privy-io/js-sdk-core
 *
 * Bundle with:
 *   npx esbuild public/privy-bridge.js \
 *     --bundle --format=esm --outfile=public/privy-bridge.bundle.js
 *
 * ⚠️  js-sdk-core is a low-level library.  Key API facts that differ from
 *    the React SDK or older docs:
 *
 *  1. DEFAULT export — `import Privy from "@privy-io/js-sdk-core"`
 *     There is NO named `PrivyClient` export.
 *
 *  2. Storage must be a plain {get,put,del,getKeys} object, NOT the raw
 *     localStorage reference.
 *
 *  3. Browser-extension wallets (MetaMask / Phantom) require a manual
 *     SIWE / SIWS handshake — there is no single `connectWallet()` helper:
 *
 *     MetaMask  →  window.ethereum   →  SIWE  (EIP-191 personal_sign)
 *     Phantom   →  window.solana     →  SIWS  (Solana signMessage)
 */

// ✅  Default import + named LocalStorage helper — NOT { PrivyClient }
import Privy, { LocalStorage } from "@privy-io/js-sdk-core";

// Named exports — wasm-bindgen binds directly to these via
// #[wasm_bindgen(module = "/public/privy-bridge.js")].
// No window.* assignment needed; the JS module system guarantees
// these are resolved before any WASM extern is called.

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------
/** @type {InstanceType<typeof Privy> | null} */
let privy = null;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function assertReady() {
    if (!privy) throw new Error("Privy not initialised — call init() first");
}

/**
 * Walk the linkedAccounts array and find the first wallet account.
 * js-sdk-core exposes the wallet inside linkedAccounts, not as a top-level
 * `user.wallet` property.
 */
function findWallet(linkedAccounts, chainType) {
    return (
        linkedAccounts?.find(
            (a) => a.type === "wallet" && (!chainType || a.chain_type === chainType)
        ) ?? null
    );
}

function serializeUser(user) {
    if (!user) return null;

    const linked = user.linkedAccounts ?? [];
    const evmWallet = findWallet(linked, "ethereum");
    const solWallet = findWallet(linked, "solana");
    // Whichever we just linked is the "active" wallet
    const wallet = evmWallet ?? solWallet ?? null;

    return {
        id: user.id,
        linkedAccounts: linked,
        wallet: wallet
            ? {
                address: wallet.address,
                chainType: wallet.chain_type,          // "ethereum" | "solana"
                walletClient: wallet.wallet_client_type ?? wallet.connector_type ?? null,
            }
            : null,
        createdAt: user.created_at ?? null,
    };
}

// ---------------------------------------------------------------------------
// Named exports — bound by wasm-bindgen via module = "/public/privy-bridge.js"
// ---------------------------------------------------------------------------

/**
 * Initialise the Privy client.
 * @param {string} appId    — from the Privy dashboard (App Settings → App ID)
 * @param {string} clientId — from the Privy dashboard (App Settings → Client ID)
 */
export async function init(appId, clientId) {
    // LocalStorage is Privy's built-in adapter — no custom {get,put,del,getKeys} needed.
    // clientId is required alongside appId for the js-sdk-core constructor.
    // There is no separate privy.init() call — the constructor handles setup.
    privy = new Privy({
        appId,
        clientId,
        storage: new LocalStorage(),
    });
}

// -----------------------------------------------------------------------------
// MetaMask — SIWE (Sign-In with Ethereum)
// -----------------------------------------------------------------------------
export async function loginWithMetaMask() {
    assertReady();

    const ethereum = window.ethereum;
    if (!ethereum) throw new Error("MetaMask not found — install the extension");

    const accounts = await ethereum.request({ method: "eth_requestAccounts" });
    const address = accounts[0];
    if (!address) throw new Error("No account returned from MetaMask");

    const chainIdHex = await ethereum.request({ method: "eth_chainId" });
    const chainId = parseInt(chainIdHex, 16);
    const caip2 = `eip155:${chainId}`;

    const siweMessage = await privy.auth.siwe.generateSiweMessage({
        address,
        chainId: caip2,
    });

    const signature = await ethereum.request({
        method: "personal_sign",
        params: [siweMessage, address],
    });

    const { user } = await privy.auth.siwe.loginWithSiwe({
        message: siweMessage,
        signature,
    });

    return serializeUser(user);
}

// -----------------------------------------------------------------------------
// Phantom — SIWS (Sign-In with Solana)
// -----------------------------------------------------------------------------
export async function loginWithPhantom() {
    assertReady();

    const solana = window.solana;
    if (!solana?.isPhantom) throw new Error("Phantom not found — install the extension");

    await solana.connect();
    const address = solana.publicKey?.toString();
    if (!address) throw new Error("No public key returned from Phantom");

    const siwsMessage = await privy.auth.siws.generateSiwsMessage({ address });

    const encodedMessage = new TextEncoder().encode(siwsMessage);
    const { signature: signatureBytes } = await solana.signMessage(encodedMessage, "utf8");

    // Convert Uint8Array → base64 string that Privy expects
    const signature = btoa(String.fromCharCode(...signatureBytes));

    const { user } = await privy.auth.siws.loginWithSiws({
        message: siwsMessage,
        signature,
    });

    return serializeUser(user);
}

// -----------------------------------------------------------------------------
// Session helpers
// -----------------------------------------------------------------------------
export async function logout() {
    assertReady();
    await privy.auth.logout();
}

export function getUser() {
    if (!privy) return null;
    return serializeUser(privy.user);
}

export function isAuthenticated() {
    return privy?.user != null;
}

export async function getAccessToken() {
    assertReady();
    return privy.getAccessToken?.() ?? null;
}