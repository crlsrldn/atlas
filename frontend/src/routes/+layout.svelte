<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="app-layout">
  <nav class="sidebar">
    <div class="logo">
      <h1>Project Atlas</h1>
    </div>
    <ul class="nav-links">
      <li><a href="/" class:active={$page.url.pathname === '/'}>Overview</a></li>
      <li><a href="/streams" class:active={$page.url.pathname === '/streams'}>Streams</a></li>
      <li><a href="/settings" class:active={$page.url.pathname === '/settings'}>Settings</a></li>
      <li><a href="/health" class:active={$page.url.pathname === '/health'}>Health</a></li>
    </ul>
  </nav>

  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(body) {
    background-color: #0d0d0d;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    margin: 0;
    padding: 0;
    height: 100vh;
    overflow: hidden;
  }

  .app-layout {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 250px;
    background: rgba(255, 255, 255, 0.03);
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    padding: 2rem 0;
    backdrop-filter: blur(20px);
  }

  .logo {
    padding: 0 2rem;
    margin-bottom: 3rem;
  }

  .logo h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
    background: linear-gradient(135deg, #fff 0%, #a5a5a5 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .nav-links {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .nav-links li {
    margin-bottom: 0.5rem;
  }

  .nav-links a {
    display: block;
    padding: 0.75rem 2rem;
    color: #888;
    text-decoration: none;
    font-size: 1rem;
    font-weight: 500;
    transition: all 0.2s ease;
    border-left: 3px solid transparent;
  }

  .nav-links a:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-links a.active {
    color: #fff;
    background: rgba(255, 255, 255, 0.08);
    border-left-color: #fff;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 3rem;
  }
</style>
