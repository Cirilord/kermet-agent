use crate::{thread::Thread, workspace::WorkspaceRecord};
use maud::{Markup, html};

use super::super::layouts;

pub fn new_thread_page(
  workspaces: &[WorkspaceRecord],
  threads: &[Thread],
  csrf_token: &str,
) -> Markup {
  let content = html! {
    div class="flex min-h-[calc(100svh-4rem-3rem)] w-full flex-col" {
      div class="mx-auto flex w-full max-w-3xl flex-1 flex-col py-8 sm:py-12" {
        div class="flex flex-1 flex-col justify-center pb-12" {
          div class="space-y-2 text-center" {
            div class="mx-auto flex size-10 items-center justify-center rounded-lg border border-border bg-card text-card-foreground" {
              i aria-hidden="true" class="size-5" data-lucide="message-square-plus" {}
            }
            h1 class="text-2xl font-semibold tracking-tight" { "Start a new thread" }
            p class="text-sm text-muted-foreground" { "Ask Kermet to work in your workspace." }
          }
        }
        div class="relative mb-3 self-start" id="workspace-selector" {
            button aria-controls="workspace-menu" aria-expanded="false" aria-haspopup="menu" class="inline-flex h-9 max-w-full items-center gap-2 rounded-md border border-input bg-transparent px-3 text-sm font-medium text-foreground shadow-xs transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" id="workspace-trigger" type="button" {
              i aria-hidden="true" class="size-4 shrink-0 text-muted-foreground" data-lucide="folder-git-2" {}
              span class="truncate" id="workspace-trigger-label" { "Select workspace" }
              i aria-hidden="true" class="size-4 shrink-0 text-muted-foreground" data-lucide="chevrons-up-down" {}
            }
            div aria-labelledby="workspace-trigger" class="absolute bottom-full left-0 z-20 mb-2 hidden w-80 overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md" id="workspace-menu" role="menu" {
              div class="px-2 py-1.5 text-xs font-medium text-muted-foreground" { "Workspaces" }
              @if workspaces.is_empty() {
                p class="px-2 py-2 text-sm text-muted-foreground" { "No saved workspaces" }
              } @else {
                @for workspace in workspaces {
                  button class="flex w-full items-center gap-2 rounded-sm px-2 py-2 text-left text-sm outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent" data-workspace-id=(workspace.id()) data-workspace-path=(workspace.path().display()) role="menuitem" type="button" {
                    i aria-hidden="true" class="size-4 shrink-0 text-muted-foreground" data-lucide="folder-git-2" {}
                    span class="truncate" { (workspace.path().display()) }
                  }
                }
              }
              div class="my-1 h-px bg-border" {}
              button class="flex w-full items-center gap-2 rounded-sm px-2 py-2 text-left text-sm outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent" id="new-workspace-trigger" role="menuitem" type="button" {
                i aria-hidden="true" class="size-4 shrink-0" data-lucide="folder-plus" {}
                "New workspace"
              }
            }
        }
        input id="workspace-id" type="hidden" {}
        div class="mb-3 hidden flex flex-1 flex-col gap-3" id="thread-transcript" aria-live="polite" {}
        form class="relative rounded-xl border border-border bg-card p-2 shadow-sm" id="new-thread-form" {
          button aria-label="Scroll to latest message" class="absolute bottom-full left-1/2 z-30 mb-3 hidden inline-flex size-9 -translate-x-1/2 items-center justify-center rounded-md border border-border bg-card text-card-foreground shadow-sm hover:bg-accent hover:text-accent-foreground" id="thread-scroll-bottom" type="button" {
            i aria-hidden="true" class="size-4" data-lucide="arrow-down" {}
          }
          label class="sr-only" for="thread-message" { "Message" }
          div class="flex items-center gap-2" {
            input autofocus autocomplete="off" class="flex h-9 min-w-0 flex-1 bg-transparent px-2 text-sm outline-none placeholder:text-muted-foreground" id="thread-message" name="message" placeholder="Ask Kermet anything..." type="text" {}
            button aria-label="Send message" class="inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground" id="thread-submit" type="submit" {
              i aria-hidden="true" class="size-4" data-lucide="arrow-up" {}
            }
          }
        }
      }
    }
    div aria-hidden="true" class="fixed inset-0 z-50 hidden bg-black/80" id="workspace-dialog-backdrop" {}
    div aria-describedby="workspace-dialog-description" aria-labelledby="workspace-dialog-title" aria-modal="true" class="fixed left-1/2 top-1/2 z-50 hidden w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-lg border border-border bg-card p-6 text-card-foreground shadow-lg" id="workspace-dialog" role="dialog" {
      div class="space-y-2" {
        h2 class="text-lg font-semibold" id="workspace-dialog-title" { "New workspace" }
        p class="text-sm text-muted-foreground" id="workspace-dialog-description" { "Register a local directory for future threads." }
      }
      div class="mt-5 space-y-2" {
        label class="text-sm font-medium" for="workspace-path" { "Folder path" }
        input autocomplete="off" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring" id="workspace-path" name="workspace-path" placeholder="/Users/name/project" readonly type="text" {}
        input id="workspace-csrf-token" type="hidden" value=(csrf_token) {}
      }
      p aria-live="polite" class="mt-2 text-sm text-muted-foreground" id="workspace-picker-status" { "Choose a local directory to register it." }
      div class="mt-6 flex justify-end gap-2" {
        button class="inline-flex h-9 items-center justify-center rounded-md border border-input bg-transparent px-3 text-sm font-medium hover:bg-accent hover:text-accent-foreground" id="workspace-dialog-cancel" type="button" { "Cancel" }
        button class="inline-flex h-9 items-center justify-center rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground" id="workspace-picker-trigger" type="button" { "Choose folder" }
      }
    }
  };
  let page = layouts::thread_page("New thread", content, threads, None);

  return page;
}
