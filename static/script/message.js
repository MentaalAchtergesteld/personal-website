document.addEventListener('click', function(e) {
    if (e.target && e.target.classList.contains('toggle-btn')) {
        const container = e.target.closest('.message');
        const content = container.querySelector('.message-content');
        
        const isCollapsed = content.classList.toggle('collapsed');
        
        e.target.innerText = isCollapsed ? "Show more" : "Show less";
    }
});
