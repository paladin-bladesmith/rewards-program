#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getClippyToolchain,
  getProgramFolders,
} from '../utils.mjs';

const channel = getClippyToolchain();
const toolchain = channel ? `+${channel}` : '';

// Lint the programs using clippy.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    await $`cd ${path.join(workingDirectory, folder)}`.quiet();
    await $`cargo ${toolchain} clippy ${process.argv.slice(3)}`;
  })
);
