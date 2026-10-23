use maud::{DOCTYPE, Markup, html};

use super::{sidebar, theme};

pub(crate) fn page(title: &str, section: Option<&str>, content: Markup) -> Markup {
  let page = html! {
    (DOCTYPE)
    html class="dark bg-background" lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
        script src="https://code.jquery.com/jquery-4.0.0.min.js" {}
        script src="https://unpkg.com/lucide@latest" {}
        (theme())
      }
      body class="min-h-screen bg-background text-foreground antialiased" {
        div class="fixed inset-0 z-40 hidden bg-foreground/80 md:hidden" id="sidebar-backdrop" {}
        div class="hub-shell flex min-h-svh w-full" id="hub-shell" {
          (sidebar())
          div class="flex min-w-0 flex-1 flex-col bg-background" {
            header class="flex h-16 shrink-0 items-center gap-2 border-b border-border px-4" {
              button aria-label="Toggle sidebar" class="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" id="sidebar-toggle" type="button" {
                i aria-hidden="true" class="size-4" data-lucide="panel-left" {}
              }
              span aria-hidden="true" class="h-4 w-px bg-border" {}
              nav aria-label="Breadcrumb" class="flex items-center gap-2 text-sm" {
                @if let Some(section) = section {
                  a class="text-muted-foreground hover:text-foreground" href="/" { "Kermet Hub" }
                  span aria-hidden="true" class="text-muted-foreground" { "/" }
                  span class="font-medium text-foreground" { (section) }
                } @else {
                  a class="font-medium text-foreground hover:text-muted-foreground" href="/" { "Kermet Hub" }
                }
              }
            }
            main class="w-full flex-1 p-6" { (content) }
          }
        }
        script {
          (maud::PreEscaped("$(function () {\n  lucide.createIcons();\n\n  const $shell = $('#hub-shell');\n  const $sidebar = $('#hub-sidebar');\n  const $backdrop = $('#sidebar-backdrop');\n  const $threadsToggle = $('#threads-toggle');\n  const $threadsMenu = $('#threads-menu');\n  const $threadsChevron = $('#threads-chevron');\n\n  function closeSidebar() {\n    $sidebar.addClass('hidden').removeClass('flex');\n    $backdrop.addClass('hidden');\n  }\n\n  function toggleThreadsMenu() {\n    const isExpanded = $threadsToggle.attr('aria-expanded') === 'true';\n    const nextState = !isExpanded;\n\n    $threadsToggle.attr('aria-expanded', nextState);\n    $threadsMenu.toggleClass('hidden', !nextState);\n    $threadsChevron.toggleClass('rotate-90', nextState);\n  }\n\n  $('#sidebar-toggle').on('click', function () {\n    if (window.matchMedia('(min-width: 768px)').matches) {\n      $shell.toggleClass('sidebar-collapsed');\n      return;\n    }\n\n    $sidebar.removeClass('hidden').addClass('flex');\n    $backdrop.removeClass('hidden');\n  });\n\n  $threadsToggle.on('click', toggleThreadsMenu);\n  $('#sidebar-close, #sidebar-backdrop').on('click', closeSidebar);\n});"))
        }
      }
    }
  };

  return page;
}
