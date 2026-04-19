import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

export default function Layout() {
  const { user, logout } = useAuth()
  const nav = useNavigate()

  const handleLogout = () => { logout(); nav('/login') }

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <div className="brand-icon">◈</div>
          <h1>World Info</h1>
          <span className="brand-tag">Developer Portal</span>
        </div>
        <nav className="sidebar-nav">
          <NavLink to="/" end className={({isActive}) => isActive ? 'nav-item active' : 'nav-item'}>
            <span className="nav-icon">⌘</span> Dashboard
          </NavLink>
          <NavLink to="/keys" className={({isActive}) => isActive ? 'nav-item active' : 'nav-item'}>
            <span className="nav-icon">⚿</span> API Keys
          </NavLink>
          <NavLink to="/config" className={({isActive}) => isActive ? 'nav-item active' : 'nav-item'}>
            <span className="nav-icon">⚙</span> Configuration
          </NavLink>
          <NavLink to="/plans" className={({isActive}) => isActive ? 'nav-item active' : 'nav-item'}>
            <span className="nav-icon">◇</span> Plans
          </NavLink>
        </nav>
        <div className="sidebar-footer">
          {user?.user && (
            <div className="user-info">
              <div className="user-avatar">{user.user.name?.[0]?.toUpperCase() || '?'}</div>
              <div className="user-details">
                <span className="user-name">{user.user.name}</span>
                <span className="user-plan">{user.user.plan?.toUpperCase()}</span>
              </div>
            </div>
          )}
          <button className="btn-logout" onClick={handleLogout}>Sign Out</button>
        </div>
      </aside>
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  )
}
