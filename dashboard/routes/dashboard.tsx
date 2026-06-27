import ConfigForm from "../islands/ConfigForm.tsx";

export default function SubscriberDashboard() {
  return (
    <div class="px-4 py-8 mx-auto max-w-screen-md min-h-screen">
      <h1 class="text-4xl font-bold mb-8">Subscriber Dashboard</h1>
      <ConfigForm projectId="atlas" />
    </div>
  );
}
