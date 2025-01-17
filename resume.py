import sqlite3
import logging
import time
import smtplib

from email.mime.text import MIMEText
from datetime import timedelta, datetime
from time import strftime, localtime

def apt_dict_to_str(apt):
    for key in ['coordinates', 'floor', 'rooms', 'size', 'postcode', 'price', 'time_to_work']:
        if apt[key] is None:
            apt[key] = 'Missing'

    if apt['time_to_work'] == 'Missing':
        apt['time_to_work'] = 0

    return f'''<b>ID</b>: {apt['id']}<br>
<b>Coordinates</b>: {apt['coordinates']}<br>
<b>Published</b>: {strftime('%d-%m-%Y %H:%M:%S', localtime(apt['published']/1000))}<br>
<b>Floor</b>: {apt['floor']}<br>
<b>Rooms</b>: {apt['rooms']}<br>
<b>Size</b>: {apt['size']}<br>
<b>Postcode</b>: {apt['postcode']}<br>
<b>URL</b>: {apt['url']}<br>
<b>Price</b>: {apt['price']}<br>
<b>Time to work</b>: {str(timedelta(seconds=apt['time_to_work']))}'''

logging.basicConfig(
    format='%(asctime)s %(levelname)-8s %(message)s',
    level=logging.INFO,
    datefmt='%Y-%m-%d %H:%M:%S')

logging.info('Starting resume')

conn = sqlite3.connect('data.db')
cursor = conn.cursor()

body = '<h1>Top 10 by price</h1>'

top10_price = {}

cursor.execute('''SELECT 
                  id, coordinates, published, floor, rooms,
                  size, postcode, price, url, json, time_to_work
                  FROM apartments WHERE
                  url NOT LIKE '%gemein%' AND
                  url NOT LIKE '%Gemein%' AND
                  price IS NOT NULL
                  ORDER BY price ASC LIMIT 10''')

for row in cursor.fetchall():
    apt = {}

    apt['id'] = row[0]
    apt['coordinates'] = row[1]
    apt['published'] = row[2]
    apt['floor'] = row[3]
    apt['rooms'] = row[4]
    apt['size'] = row[5]
    apt['postcode'] = row[6]
    apt['price'] = row[7]
    apt['url'] = row[8]
    apt['json'] = row[9]
    apt['time_to_work'] = row[10]

    body = body + apt_dict_to_str(apt) + '<br><br>'

    top10_price[row[0]] = apt

body = body + '<h1>Top 10 by proximity</h1>'

top10_distance = {}

cursor.execute('''SELECT 
                  id, coordinates, published, floor, rooms,
                  size, postcode, price, url, json, time_to_work
                  FROM apartments WHERE
                  coordinates IS NOT NULL AND
                  url NOT LIKE '%gemein%' AND
                  url NOT LIKE '%Gemein%' AND
                  price IS NOT NULL
                  ORDER BY time_to_work ASC LIMIT 10''')

for row in cursor.fetchall():
    apt = {}

    apt['id'] = row[0]
    apt['coordinates'] = row[1]
    apt['published'] = row[2]
    apt['floor'] = row[3]
    apt['rooms'] = row[4]
    apt['size'] = row[5]
    apt['postcode'] = row[6]
    apt['price'] = row[7]
    apt['url'] = row[8]
    apt['json'] = row[9]
    apt['time_to_work'] = row[10]

    body = body + apt_dict_to_str(apt) + '<br><br>'

    top10_distance[row[0]] = apt

body_mutual = ''

for key_price in top10_price.keys():
    if key_price in top10_distance:
        body_mutual = body_mutual + apt_dict_to_str(apt) + '<br><br>'

if body_mutual != '':
    body = '<h1>Mutual</h1>' + body_mutual

msg = MIMEText(body, 'html')
msg['Subject'] = 'Daily resume ' + datetime.now().strftime('%d/%m')
msg['From'] = ''
msg['To'] = ''
with smtplib.SMTP_SSL('smtp.gmail.com', 465) as smtp_server:
    smtp_server.login(msg['From'], '')
    smtp_server.sendmail(msg['From'], msg['To'], msg.as_string())
