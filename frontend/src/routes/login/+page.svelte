<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, auth } from '$lib/api';

  let username: string = '';
  let password: string = '';
  let error: string = '';
  let loading: boolean = false;

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    error = '';
    loading = true;

    try {
      const response = await api.post('/api/auth/login', {
        username,
        password
      });

      if (response.ok) {
        const data = await response.json();
        $auth.login(data.token, data.user);
        goto('/');
      } else {
        const err = await response.json().catch(() => ({}));
        error = err.message || 'Login failed. Please check your credentials.';
      }
    } catch (e) {
      error = 'Network error. Please try again.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="auth-page">
  <div class="auth-container">
    <div class="auth-header">
      <h1>Welcome Back</h1>
      <p>Sign in to your account to continue</p>
    </div>

    {#if error}
      <div class="alert alert-error">
        {error}
      </div>
    {/if}

    <form class="auth-form" on:submit={handleLogin}>
      <div class="form-group">
        <label for="username">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          placeholder="Enter your username"
          required
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          placeholder="Enter your password"
          required
          disabled={loading}
        />
      </div>

      <button type="submit" class="btn btn-primary" disabled={loading}>
        {loading ? 'Signing in...' : 'Sign In'}
      </button>
    </form>

    <div class="auth-footer">
      <p>Don't have an account? <a href="/register">Sign up</a></p>
    </div>
  </div>
</div>

<style>
  .auth-page {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 70vh;
    padding: 20px;
  }

  .auth-container {
    width: 100%;
    max-width: 420px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 12px;
    padding: 32px;
  }

  .auth-header {
    text-align: center;
    margin-bottom: 24px;
  }

  .auth-header h1 {
    margin: 0 0 8px 0;
    font-size: 28px;
    font-weight: 600;
    color: #fff;
  }

  .auth-header p {
    margin: 0;
    color: #858585;
    font-size: 14px;
  }

  .alert {
    padding: 12px 16px;
    border-radius: 6px;
    margin-bottom: 20px;
    font-size: 14px;
  }

  .alert-error {
    background: rgba(244, 67, 54, 0.15);
    border: 1px solid rgba(244, 67, 54, 0.3);
    color: #f44336;
  }

  .auth-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 14px;
    font-weight: 500;
    color: #ccc;
  }

  .form-group input {
    padding: 12px 16px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 6px;
    color: #fff;
    font-size: 14px;
    transition: border-color 0.2s;
  }

  .form-group input:focus {
    outline: none;
    border-color: #007acc;
  }

  .form-group input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn {
    padding: 14px 24px;
    border-radius: 6px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #007acc;
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #005a9e;
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .auth-footer {
    text-align: center;
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid #333;
  }

  .auth-footer p {
    margin: 0;
    color: #858585;
    font-size: 14px;
  }

  .auth-footer a {
    color: #007acc;
    text-decoration: none;
    font-weight: 500;
  }

  .auth-footer a:hover {
    text-decoration: underline;
  }
</style>
