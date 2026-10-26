use maud::{Markup, html};

pub(crate) fn sidebar(threads_active: bool) -> Markup {
  let threads_class = if threads_active {
    "sidebar-menu-button flex h-8 w-full items-center gap-2 overflow-hidden rounded-md bg-sidebar-accent p-2 text-sm font-medium text-sidebar-accent-foreground"
  } else {
    "sidebar-menu-button flex h-8 w-full items-center gap-2 overflow-hidden rounded-md p-2 text-sm font-medium hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
  };
  let sidebar = html! {
    aside id="hub-sidebar" class="hub-sidebar fixed inset-y-0 left-0 z-50 hidden w-64 shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-[width] duration-200 ease-linear md:sticky md:inset-y-auto md:top-0 md:flex md:h-svh md:w-56 md:self-start" {
      div class="flex flex-col gap-2 p-2" {
        a class="sidebar-menu-button flex h-12 w-full items-center gap-2 overflow-hidden rounded-md p-2 text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground" href="/" {
          span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sm font-semibold text-sidebar-primary-foreground" { "K" }
          span class="sidebar-label grid min-w-0 flex-1 text-left leading-tight" {
            span class="truncate font-medium" { "Kermet" }
            span class="truncate text-xs text-muted-foreground" { "Local agent" }
          }
        }
        button aria-label="Close sidebar" class="absolute right-2 top-3 inline-flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground md:hidden" id="sidebar-close" type="button" {
          i aria-hidden="true" class="size-4" data-lucide="x" {}
        }
      }

      div class="flex min-h-0 flex-1 flex-col overflow-auto" {
        div class="flex w-full min-w-0 flex-col p-2" {
          p class="sidebar-group-label flex h-8 items-center rounded-md px-2 text-xs font-medium text-muted-foreground" { "Platform" }
          nav aria-label="Platform" class="flex w-full min-w-0 flex-col" {
            a class=(threads_class) href="/threads" {
              i aria-hidden="true" class="size-4 shrink-0" data-lucide="messages-square" {}
              span class="sidebar-label truncate" { "Threads" }
            }
            /*
            span aria-disabled="true" class="sidebar-menu-button flex h-8 cursor-not-allowed items-center gap-2 overflow-hidden rounded-md p-2 text-sm text-muted-foreground opacity-60" {
              i aria-hidden="true" class="size-4 shrink-0" data-lucide="bot" {}
              span class="sidebar-label truncate" { "Models" }
            }
            span aria-disabled="true" class="sidebar-menu-button flex h-8 cursor-not-allowed items-center gap-2 overflow-hidden rounded-md p-2 text-sm text-muted-foreground opacity-60" {
              i aria-hidden="true" class="size-4 shrink-0" data-lucide="book-open" {}
              span class="sidebar-label truncate" { "Documentation" }
            }
            span aria-disabled="true" class="sidebar-menu-button flex h-8 cursor-not-allowed items-center gap-2 overflow-hidden rounded-md p-2 text-sm text-muted-foreground opacity-60" {
              i aria-hidden="true" class="size-4 shrink-0" data-lucide="settings-2" {}
              span class="sidebar-label truncate" { "Settings" }
            }
            */
          }
        }

        /*
        div class="sidebar-projects mt-2 flex w-full min-w-0 flex-col p-2" {
          p class="flex h-8 items-center rounded-md px-2 text-xs font-medium text-muted-foreground" { "Projects" }
          div class="flex w-full min-w-0 flex-col" {
            a class="sidebar-menu-button flex h-8 w-full items-center gap-2 overflow-hidden rounded-md p-2 text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground" href="/" {
              span class="flex size-4 shrink-0 items-center justify-center rounded-sm border border-sidebar-border text-[10px] font-semibold" { "K" }
              span class="sidebar-label truncate" { "Kermet Agent" }
              i aria-hidden="true" class="sidebar-label ml-auto size-4 text-muted-foreground" data-lucide="ellipsis" {}
            }
          }
        }
        */
      }

      /*
      div class="flex flex-col gap-2 border-t border-sidebar-border p-2" {
        div class="sidebar-menu-button flex h-12 items-center gap-2 overflow-hidden rounded-md p-2" {
          span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-xs font-semibold text-muted-foreground" { "KH" }
          span class="sidebar-footer-label grid min-w-0 flex-1 text-left text-sm leading-tight" {
            span class="truncate font-medium" { "Kermet Hub" }
            span class="truncate text-xs text-muted-foreground" { "Local-only" }
          }
          i aria-hidden="true" class="sidebar-footer-label size-4 text-muted-foreground" data-lucide="chevrons-up-down" {}
        }
      }
      */
    }
  };

  return sidebar;
}
