#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getRustfmtToolchain,
  getProgramFolders,
} from '../utils.mjs';

const channel = getRustfmtToolchain();
const toolchain = channel ? `+${channel}` : '';

// Format the programs.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    await $`cd ${path.join(workingDirectory, folder)}`.quiet();
    await $`cargo ${toolchain} fmt ${process.argv.slice(3)}`;
  })
);
