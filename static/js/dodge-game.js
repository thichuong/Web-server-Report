// Dodge Game - Mini game for loading screen
class DodgeGame {
    constructor(canvasId, scoreElementId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.scoreElement = document.getElementById(scoreElementId);
        
        this.gameRunning = false;
        this.score = 0;
        this.speed = 2;
        this.lastObstacleTime = 0;
        this.gameTime = 0;
        
        // Theme colors
        this.updateThemeColors();
        
        // Player object
        this.player = {
            x: this.canvas.width / 2 - 10,
            y: this.canvas.height - 50,
            width: 20,
            height: 20,
            direction: Math.random() < 0.5 ? -1 : 1, // Bắt đầu ngẫu nhiên trái/phải
            speed: 3
        };
        
        // Obstacles array
        this.obstacles = [];
        
        // Bind methods
        this.handleCanvasClick = this.handleCanvasClick.bind(this);
        this.gameLoop = this.gameLoop.bind(this);
        
        // Listen for theme changes
        this.observeThemeChanges();
    }
    
    updateThemeColors() {
        const isDarkTheme = document.documentElement.getAttribute('data-theme') === 'dark';
        
        this.colors = {
            background: isDarkTheme ? '#000000' : '#ffffff',
            player: isDarkTheme ? '#ffffff' : '#000000',
            obstacle: isDarkTheme ? '#ffffff' : '#000000',
            text: isDarkTheme ? '#ffffff' : '#666666',
            border: isDarkTheme ? '#333333' : '#cccccc'
        };
    }
    
    observeThemeChanges() {
        // Create a MutationObserver to watch for theme changes
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
                    this.updateThemeColors();
                }
            });
        });
        
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['data-theme']
        });
    }
    
    start() {
        this.gameRunning = true;
        this.score = 0;
        this.speed = 2;
        this.lastObstacleTime = 0;
        this.gameTime = 0;
        this.obstacles = [];
        
        // Update colors for current theme
        this.updateThemeColors();
        
        // Reset player position và random hướng ban đầu
        this.player.x = this.canvas.width / 2 - 10;
        this.player.y = this.canvas.height - 50;
        this.player.direction = Math.random() < 0.5 ? -1 : 1;
        
        this.scoreElement.textContent = this.score;
        
        // Add click event to canvas
        this.canvas.addEventListener('click', this.handleCanvasClick);
        
        // Start the game loop
        this.gameLoop();
    }
    
    stop() {
        this.gameRunning = false;
        this.canvas.removeEventListener('click', this.handleCanvasClick);
    }
    
    handleCanvasClick() {
        if (this.gameRunning) {
            // Đổi hướng: trái thành phải, phải thành trái
            this.player.direction = this.player.direction === 1 ? -1 : 1;
        }
    }
    
    createObstacle() {
        // Tạo vật cản hình vuông với kích thước khác nhau
        const minSize = 25;
        const maxSize = 60;
        const size = minSize + Math.random() * (maxSize - minSize);
        
        // Random vị trí x trong toàn bộ khung trên (trừ margins)
        const margin = 10;
        const maxX = this.canvas.width - size - margin;
        const x = margin + Math.random() * (maxX - margin);
        
        this.obstacles.push({
            x: x,
            y: -size,
            width: size,
            height: size, // Hình vuông
            color: this.colors.obstacle
        });
    }
    
    updatePlayer() {
        // Di chuyển player theo hướng
        this.player.x += this.player.direction * this.player.speed;
        
        // Kiểm tra chạm biên và trừ điểm
        if (this.player.x <= 0 || this.player.x + this.player.width >= this.canvas.width) {
            // Trừ điểm khi chạm biên
            this.score = Math.max(0, this.score - 20);
            this.scoreElement.textContent = this.score;
            
            // Giữ player trong canvas
            if (this.player.x <= 0) this.player.x = 1;
            if (this.player.x + this.player.width >= this.canvas.width) {
                this.player.x = this.canvas.width - this.player.width - 1;
            }
            
            // Đổi hướng khi chạm biên
            this.player.direction *= -1;
        }
    }
    
    updateObstacles() {
        // Di chuyển vật cản xuống
        for (let i = this.obstacles.length - 1; i >= 0; i--) {
            this.obstacles[i].y += this.speed;
            
            // Update obstacle color for current theme
            this.obstacles[i].color = this.colors.obstacle;
            
            // Xóa vật cản đã qua đáy
            if (this.obstacles[i].y > this.canvas.height) {
                this.obstacles.splice(i, 1);
                this.score += 10;
                this.scoreElement.textContent = this.score;
            }
        }
        
        // Tạo vật cản mới
        const obstacleInterval = Math.max(800, 1500 - (this.speed * 50));
        if (this.gameTime - this.lastObstacleTime > obstacleInterval) {
            this.createObstacle();
            this.lastObstacleTime = this.gameTime;
        }
    }
    
    checkCollision() {
        for (let obstacle of this.obstacles) {
            // Tính toán collision cho hình kim cương
            const obstacleCenterX = obstacle.x + obstacle.width / 2;
            const obstacleCenterY = obstacle.y + obstacle.height / 2;
            const obstacleRadius = obstacle.width / 2;
            
            // Tính toán player bounds
            const playerCenterX = this.player.x + this.player.width / 2;
            const playerCenterY = this.player.y + this.player.height / 2;
            const playerHalfWidth = this.player.width / 2;
            const playerHalfHeight = this.player.height / 2;
            
            // Kiểm tra collision với kim cương bằng cách kiểm tra 4 cạnh của kim cương
            if (this.isPointInDiamond(this.player.x, this.player.y, obstacleCenterX, obstacleCenterY, obstacleRadius) ||
                this.isPointInDiamond(this.player.x + this.player.width, this.player.y, obstacleCenterX, obstacleCenterY, obstacleRadius) ||
                this.isPointInDiamond(this.player.x, this.player.y + this.player.height, obstacleCenterX, obstacleCenterY, obstacleRadius) ||
                this.isPointInDiamond(this.player.x + this.player.width, this.player.y + this.player.height, obstacleCenterX, obstacleCenterY, obstacleRadius)) {
                return true;
            }
            
            // Kiểm tra ngược lại: center của kim cương có trong player không
            if (playerCenterX - playerHalfWidth <= obstacleCenterX && 
                obstacleCenterX <= playerCenterX + playerHalfWidth &&
                playerCenterY - playerHalfHeight <= obstacleCenterY && 
                obstacleCenterY <= playerCenterY + playerHalfHeight) {
                return true;
            }
        }
        return false;
    }
    
    // Kiểm tra điểm có nằm trong kim cương không
    isPointInDiamond(px, py, diamondCenterX, diamondCenterY, radius) {
        // Sử dụng Manhattan distance cho kim cương
        const dx = Math.abs(px - diamondCenterX);
        const dy = Math.abs(py - diamondCenterY);
        return (dx / radius + dy / radius) <= 1;
    }
    
    draw() {
        // Nền theo theme
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // Vẽ player (hình vuông bình thường)
        this.ctx.fillStyle = this.colors.player;
        this.ctx.fillRect(this.player.x, this.player.y, this.player.width, this.player.height);
        
        // Vẽ vật cản (hình thoi với góc nhọn xuống)
        for (let obstacle of this.obstacles) {
            this.drawDiamond(
                obstacle.x + obstacle.width / 2, // center x
                obstacle.y + obstacle.height / 2, // center y
                obstacle.width / 2, // radius
                obstacle.color
            );
        }
        
        // Vẽ chỉ thị hướng
        this.ctx.fillStyle = this.colors.text;
        this.ctx.font = '14px Arial';
        this.ctx.textAlign = 'center';
        const dirText = this.player.direction === -1 ? '←' : '→';
        this.ctx.fillText(dirText, this.player.x + this.player.width/2, this.player.y - 5);
        
        // Vẽ viền canvas
        this.ctx.strokeStyle = this.colors.border;
        this.ctx.lineWidth = 2;
        this.ctx.strokeRect(0, 0, this.canvas.width, this.canvas.height);
    }
    
    // Hàm vẽ hình thoi với góc nhọn xuống dưới
    drawDiamond(centerX, centerY, radius, color) {
        this.ctx.fillStyle = color;
        this.ctx.beginPath();
        
        // Tọa độ 4 đỉnh của hình thoi (diamond), góc nhọn ở dưới
        this.ctx.moveTo(centerX, centerY - radius); // Đỉnh trên
        this.ctx.lineTo(centerX + radius, centerY); // Đỉnh phải
        this.ctx.lineTo(centerX, centerY + radius); // Đỉnh dưới (nhọn)
        this.ctx.lineTo(centerX - radius, centerY); // Đỉnh trái
        this.ctx.closePath();
        
        this.ctx.fill();
    }
    
    gameLoop() {
        if (!this.gameRunning) return;
        
        this.gameTime += 16; // ~60fps
        
        // Tăng tốc theo điểm số (mỗi 100 điểm tăng 0.5 tốc độ)
        this.speed = 2 + (this.score / 200);
        
        this.updatePlayer();
        this.updateObstacles();
        
        // Kiểm tra va chạm
        if (this.checkCollision()) {
            // Game over - khởi động lại
            this.obstacles = [];
            this.player.x = this.canvas.width / 2 - 10;
            this.player.direction = Math.random() < 0.5 ? -1 : 1; // Random hướng mới
            this.score = Math.max(0, this.score - 50);
            this.scoreElement.textContent = this.score;
        }
        
        this.draw();
        
        requestAnimationFrame(this.gameLoop);
    }
}

// Export for use in other files
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DodgeGame;
}
