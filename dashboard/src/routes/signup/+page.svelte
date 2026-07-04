<script lang="ts">
  import AuthForm from '$lib/components/AuthForm.svelte';
</script>

<svelte:head>
  <title>Sign Up — Atlas</title>
</svelte:head>

<div class="flex-grow flex items-center justify-center p-4 min-h-[80vh]">
  <AuthForm type="signup" />
</div>
