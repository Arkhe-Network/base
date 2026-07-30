import Lake
open Lake DSL

package CathedralArkhe where
  -- add package configuration options here

lean_lib CathedralArkhe where
  -- add library configuration options here

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.7.0"
