export default function AdminDashboard() {
  return (
    <div class="px-4 py-8 mx-auto max-w-screen-md min-h-screen">
      <h1 class="text-4xl font-bold mb-8">Admin Dashboard</h1>
      
      <div class="grid grid-cols-2 gap-4">
        <div class="bg-white p-6 rounded-lg shadow-sm border">
          <h2 class="text-lg font-medium text-gray-500 mb-2">Total Users</h2>
          <p class="text-4xl font-bold">142</p>
        </div>
        
        <div class="bg-white p-6 rounded-lg shadow-sm border">
          <h2 class="text-lg font-medium text-gray-500 mb-2">Streams Resolved</h2>
          <p class="text-4xl font-bold">8,432</p>
        </div>
      </div>
    </div>
  );
}
