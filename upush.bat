echo off
echo "This upush.bat file is *not* safe - extremely rudimental"
git add .
git commit -m %1
git pull
git push
