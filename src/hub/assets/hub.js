$(function () {
  lucide.createIcons();

  const $shell = $('#hub-shell');
  const $sidebar = $('#hub-sidebar');
  const $backdrop = $('#sidebar-backdrop');
  const $threadsToggle = $('#threads-toggle');
  const $threadsMenu = $('#threads-menu');
  const $threadsChevron = $('#threads-chevron');

  function closeSidebar() {
    $sidebar.addClass('hidden').removeClass('flex');
    $backdrop.addClass('hidden');
  }

  function toggleThreadsMenu() {
    const isExpanded = $threadsToggle.attr('aria-expanded') === 'true';
    const nextState = !isExpanded;

    $threadsToggle.attr('aria-expanded', nextState);
    $threadsMenu.toggleClass('hidden', !nextState);
    $threadsChevron.toggleClass('rotate-90', nextState);
  }

  $('#sidebar-toggle').on('click', function () {
    if (window.matchMedia('(min-width: 768px)').matches) {
      $shell.toggleClass('sidebar-collapsed');
      return;
    }

    $sidebar.removeClass('hidden').addClass('flex');
    $backdrop.removeClass('hidden');
  });

  $threadsToggle.on('click', toggleThreadsMenu);
  $('#sidebar-close, #sidebar-backdrop').on('click', closeSidebar);
});
