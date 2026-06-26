<script lang="ts">
  // Welcome to Project Atlas overview
</script>

<div class="header">
  <h2>Overview</h2>
  <p>Your media intelligence layer is active.</p>
</div>

<div class="cards">
  <div class="card">
    <div class="icon">🎬</div>
    <h3>Smart Play</h3>
    <p>Automatically resolving the best sources for your watch sessions.</p>
  </div>
  
  <div class="card">
    <div class="icon">⚡</div>
    <h3>Fast & Private</h3>
    <p>Everything runs securely within your configured providers.</p>
  </div>

  <div class="card highlight">
    <div class="icon">⚙️</div>
    <h3>Get Started</h3>
    <p>Head over to Settings to configure your TorBox API key and adjust playback preferences.</p>
    <a href="/settings" class="btn">Go to Settings</a>
  </div>
</div>

<style>
  .header {
    margin-bottom: 3rem;
  }

  h2 {
    font-size: 2.5rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }

  .header p {
    color: #888;
    font-size: 1.1rem;
    margin: 0;
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
  }

  .card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    padding: 2rem;
    transition: transform 0.2s ease, background 0.2s ease;
  }

  .card:hover {
    transform: translateY(-5px);
    background: rgba(255, 255, 255, 0.05);
  }

  .card.highlight {
    background: linear-gradient(145deg, rgba(255,255,255,0.08) 0%, rgba(255,255,255,0.02) 100%);
    border-color: rgba(255,255,255,0.2);
  }

  .icon {
    font-size: 2rem;
    margin-bottom: 1rem;
  }

  h3 {
    font-size: 1.3rem;
    margin: 0 0 0.5rem 0;
  }

  .card p {
    color: #a0a0a0;
    line-height: 1.5;
    margin-bottom: 1.5rem;
  }

  .btn {
    display: inline-block;
    background: #fff;
    color: #000;
    text-decoration: none;
    padding: 0.6rem 1.2rem;
    border-radius: 2rem;
    font-weight: 600;
    font-size: 0.9rem;
    transition: transform 0.2s ease;
  }

  .btn:hover {
    transform: scale(1.05);
  }
</style>
