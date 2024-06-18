#!/usr/bin/env zx
import "zx/globals";
import * as k from "kinobi";
import { execSync } from "child_process";
import { rootNodeFromAnchor } from "@kinobi-so/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@kinobi-so/renderers-js";
import { renderVisitor as renderRustVisitor } from "@kinobi-so/renderers-rust";
import { getAllProgramIdls } from "./utils.mjs";

// Instanciate Kinobi.
const [idl, ...additionalIdls] = getAllProgramIdls().map(idl => rootNodeFromAnchor(require(idl)))
const kinobi = k.createFromRoot(idl, additionalIdls);

const ciDir = path.join(__dirname, "..", "ci");

// Update programs.
kinobi.update(
  k.updateProgramsVisitor({
    "paladinRewardsProgram": { name: "rewards" },
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, "..", "clients", "js");
kinobi.accept(
  renderJavaScriptVisitor(path.join(jsClient, "src", "generated"), { 
    prettier: require(path.join(jsClient, ".prettierrc.json"))
  })
);

// Render Rust.
const rustClient = path.join(__dirname, "..", "clients", "rust");
const rustNightly = path.join(ciDir, "rust-nightly.sh");
const toolchain = execSync(`bash ${rustNightly}`).toString().trim().split('\n').pop();
kinobi.accept(
  renderRustVisitor(path.join(rustClient, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain,
  })
);
