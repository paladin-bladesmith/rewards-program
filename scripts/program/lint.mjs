#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getClippyToolchain,
  getProgramFolders,
} from '../utils.mjs';

// Lint the programs using clippy.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    await $`cd ${path.join(workingDirectory, folder)}`.quiet();
    await $`cargo ${getClippyToolchain()} clippy ${process.argv.slice(3)}`;
  })
);
