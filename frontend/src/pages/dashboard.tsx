import Sidebar from '../components/Sidebar';

const Dashboard = () => {
    return(
        <div className="flex bg-gray-50 min-h-screen">
            <Sidebar />
            <main className="flex-1 p-8">
                <h1 className="text-3xl font-bold text-gray-800">Dashboard</h1>
                <p className="text-gray-600 mt-2">Welcome to your Eclipta dashboard.</p>
            </main>
        </div>
    )
}

export default Dashboard;