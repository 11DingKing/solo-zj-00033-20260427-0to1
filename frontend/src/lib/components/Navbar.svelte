<script lang="ts">
  import { page, goto } from '$app/stores';
  import { auth } from '$lib/api';
  let searchQuery: string = '';

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    if (searchQuery.trim()) {
      goto(`/search?q=${encodeURIComponent(searchQuery)}`);
      searchQuery = '';
    }
  }

  function logout() {
    $auth.logout();
    goto('/');
  }
</script>

<nav class="navbar">
  <div class="nav-container">
    <a href="/" class="logo">
      <span class="logo-icon">{'{'}</span>
      <span class="logo-text">CodeShare</span>
      <span class="logo-icon">{'}'}</span>
    </a>

    <div class="nav-links">
      <a href="/" class="nav-link" class:active={$page.url.pathname === '/'}>Home</a>
      <a href="/hot" class="nav-link" class:active={$page.url.pathname === '/hot'}>Hot</a>
      <a href="/latest" class="nav-link" class:active={$page.url.pathname === '/latest'}>Latest</a>
    </div>

    <form class="search-form" on:submit={handleSearch}>
      <input 
        type="text" 
        bind:value={searchQuery}
        placeholder="Search snippets..."
        class="search-input"
      />
      <button type="submit" class="search-btn">🔍</button>
    </form>

    <div class="nav-actions">
      {#if $auth.token && $auth.user}
        <a href="/new" class="btn btn-primary">+ New Snippet</a>
        <div class="user-menu">
          <span class="username">{$auth.user.display_name || $auth.user.username}</span>
          <div class="dropdown">
            <a href={`/user/${$auth.user.id}`} class="dropdown-item">Profile</a>
            <a href={`/user/${$auth.user.id}?tab=forks`} class="dropdown-item">Forks</a>
            <button class="dropdown-item logout-btn" on:click={logout}>Logout</button>
          </div>
        </div>
      {:else}
        <a href="/login" class="btn btn-secondary">Login</a>
        <a href="/register" class="btn btn-primary">Register</a>
      {/if}
    </div>
  </div>
</nav>

<style>
  .navbar {
    background: #252526;
    border-bottom: 1px solid #333;
    position: sticky;
    top: 0;
    z-index: 1000;
  }

  .nav-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 20px;
    display: flex;
    align-items: center;
    gap: 24px;
    height: 54px;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 4px;
    text-decoration: none;
    color: #fff;
    font-weight: 600;
    font-size: 18px;
  }

  .logo-icon {
    color: #007acc;
    font-size: 22px;
  }

  .nav-links {
    display: flex;
    gap: 8px;
  }

  .nav-link {
    color: #ccc;
    text-decoration: none;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 14px;
    transition: all 0.2s;
  }

  .nav-link:hover {
    background: #333;
    color: #fff;
  }

  .nav-link.active {
    background: #007acc;
    color: #fff;
  }

  .search-form {
    flex: 1;
    max-width: 400px;
    display: flex;
    position: relative;
  }

  .search-input {
    flex: 1;
    padding: 8px 36px 8px 12px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 4px;
    color: #fff;
    font-size: 14px;
  }

  .search-input:focus {
    outline: none;
    border-color: #007acc;
  }

  .search-btn {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    font-size: 14px;
    color: #858585;
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
    text-decoration: none;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #007acc;
    color: #fff;
  }

  .btn-primary:hover {
    background: #005a9e;
  }

  .btn-secondary {
    background: #333;
    color: #ccc;
  }

  .btn-secondary:hover {
    background: #444;
    color: #fff;
  }

  .user-menu {
    position: relative;
  }

  .username {
    color: #ccc;
    cursor: pointer;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 14px;
  }

  .username:hover {
    background: #333;
    color: #fff;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    background: #252526;
    border: 1px solid #333;
    border-radius: 6px;
    min-width: 160px;
    margin-top: 4px;
    display: none;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .user-menu:hover .dropdown {
    display: block;
  }

  .dropdown-item {
    display: block;
    padding: 10px 16px;
    color: #ccc;
    text-decoration: none;
    font-size: 14px;
    background: none;
    border: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
  }

  .dropdown-item:hover {
    background: #333;
    color: #fff;
  }

  .logout-btn {
    color: #f44336;
  }

  .logout-btn:hover {
    background: rgba(244, 67, 54, 0.1);
  }
</style>
