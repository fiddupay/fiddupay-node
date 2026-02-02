module.exports = {
  apps: [
    {
      name: 'fiddupay-backend',
      script: './start.sh',
      cwd: '/home/vibes/crypto-payment-gateway/backend',
      instances: 1,
      exec_mode: 'fork',
      watch: false,
      max_memory_restart: '1G',
      error_file: '/var/log/fiddupay/error.log',
      out_file: '/var/log/fiddupay/out.log',
      log_file: '/var/log/fiddupay/combined.log',
      time: true,
      autorestart: true,
      max_restarts: 10,
      min_uptime: '10s',
      restart_delay: 4000,
      env: {
        RUST_LOG: 'info',
        RUST_BACKTRACE: '1'
      }
    }
  ]
};
