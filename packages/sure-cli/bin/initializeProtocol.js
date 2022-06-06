"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const nodewallet_1 = __importDefault(require("@project-serum/anchor/dist/cjs/nodewallet"));
const web3_js_1 = require("@solana/web3.js");
const sdk_1 = require("@sure/sdk");
function run() {
    return __awaiter(this, void 0, void 0, function* () {
        const keypair = web3_js_1.Keypair.fromSecretKey(Buffer.from(JSON.parse(require('fs').readFileSync(process.env.WALLET, {
            encoding: 'utf-8',
        }))));
        const wallet = new nodewallet_1.default(keypair);
        const network = process.env.NETWORK;
        const connection = new web3_js_1.Connection(network, {});
        const sureSDK = sdk_1.SureSdk.init(connection, wallet);
        yield sureSDK.protocol.initializeProtocol();
    });
}
run().catch((err) => {
    console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
    console.error(err.stack);
    process.exit(1);
});
//# sourceMappingURL=initializeProtocol.js.map