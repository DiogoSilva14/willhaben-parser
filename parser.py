import requests
import json
import sqlite3
import logging
import time
import random
import smtplib

from email.mime.text import MIMEText
from datetime import timedelta
from time import strftime, localtime
from bs4 import BeautifulSoup

filters = {
    'price': {
        'min': 500,
        'max': 850
    },
    'size': {
       'min': 39,
       'max': 200
    }
}

def parse_apt_dict(json_obj):
    ret = {}
    ret['json'] = json.dumps(json_obj)
    ret['id'] = int(json_obj['id'])

    ret['coordinates'] = None
    ret['published'] = None
    ret['floor'] = None
    ret['rooms'] = None
    ret['size'] = None
    ret['postcode'] = None
    ret['price'] = None
    ret['url'] = None
    ret['json'] = None
    ret['time_to_work'] = None

    attributes = json_obj['attributes']['attribute']

    for attr in attributes:
        key = attr['name']
        value = attr['values'][0]

        if key == 'COORDINATES':
            ret['coordinates'] = value
        elif key == 'PUBLISHED':
            ret['published'] = int(value)
        elif key == 'FLOOR':
            ret['floor'] = value
        elif key == 'NUMBER_OF_ROOMS':
            ret['rooms'] = int(value)
        elif key == 'ESTATE_SIZE/LIVING_AREA':
            ret['size'] = int(value)
        elif key == 'POSTCODE':
            ret['postcode'] = int(value)
        elif key == 'RENT/PER_MONTH_LETTINGS':
            ret['price'] = float(value)
        elif key == 'SEO_URL':
            ret['url'] = 'https://www.willhaben.at/iad/' + value

    return ret

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

URL = 'https://www.willhaben.at/iad/immobilien/mietwohnungen/wien'
API_KEY = ''
API_URL = 'https://maps.googleapis.com/maps/api/distancematrix/json'
REQUEST_PARAMS = {'key': API_KEY, 'units': 'metric', 'mode': 'transit', 'destinations': 'Karlsplatz, Wien', 'departure_time': '1737014400'}

logging.basicConfig(
    format='%(asctime)s %(levelname)-8s %(message)s',
    level=logging.INFO,
    datefmt='%Y-%m-%d %H:%M:%S')

logging.info('Starting parser')

conn = sqlite3.connect('data.db')
cursor = conn.cursor()

cursor.execute('''CREATE TABLE IF NOT EXISTS apartments
                (id INTEGER PRIMARY KEY,
                 coordinates TEXT,
                 published INTEGER,
                 floor TEXT,
                 rooms INTEGER,
                 size INTEGER,
                 postcode INTEGER,
                 price REAL,
                 url TEXT,
                 json TEXT,
                 time_to_work INTEGER)''')

returned_apartments = {}
page_id = 1

while True:
    if page_id != 1:
        time.sleep(random.randint(10,15))

    logging.info(f'Requesting page number {page_id}')

    page = requests.get(URL, params={
        'page': page_id,
        'rows': 90,
        'areaId': 900, # Wien
        'PROPERTY_TYPE': 110,
        'PROPERTY_TYPE': 105,
        'PROPERTY_TYPE': 3,
        'PRICE_FROM': filters['price']['min'],
        'PRICE_TO': filters['price']['max'],
        'ESTATE_SIZE/LIVING_AREA_FROM': filters['size']['min'],
        'ESTATE_SIZE/LIVING_AREA_TO': filters['size']['max']
    })
    page_id = page_id + 1

    soup = BeautifulSoup(page.content, 'html.parser')
    search_result = json.loads(soup.find(id='__NEXT_DATA__').contents[0])['props']['pageProps']['searchResult']
    rows_returned = search_result['rowsReturned']

    logging.info(f'Rows returned: {rows_returned}')

    if rows_returned == 0:
        break

    for post in search_result['advertSummaryList']['advertSummary']:
        apt = parse_apt_dict(post)
        if 'gemeinde' in apt['url'] or 'Gemeinde' in apt['url']:
            continue
        returned_apartments[apt['id']] = apt

logging.info(f'Retrieved {len(returned_apartments)} apartments')

stored_apartments = {}

cursor.execute('''SELECT 
                  id, coordinates, published, floor, rooms,
                  size, postcode, price, url, json, time_to_work
                  FROM apartments''')

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

    stored_apartments[row[0]] = apt

added_ids = []
removed_ids = []
updated_ids_fields = {}

logging.info('Checking new entries')

for returned_id in returned_apartments.keys():
    if returned_id in stored_apartments.keys():
        continue

    logging.info(f'New apartment {returned_id}')

    cursor.execute('''INSERT INTO apartments(
                      id, coordinates, published, floor, rooms,
                      size, postcode, price, url, json, time_to_work)
                      VALUES(?,?,?,?,?,?,?,?,?,?,?)''', (
                   returned_apartments[returned_id]['id'],
                   returned_apartments[returned_id]['coordinates'],
                   returned_apartments[returned_id]['published'],
                   returned_apartments[returned_id]['floor'],
                   returned_apartments[returned_id]['rooms'],
                   returned_apartments[returned_id]['size'],
                   returned_apartments[returned_id]['postcode'],
                   returned_apartments[returned_id]['price'],
                   returned_apartments[returned_id]['url'],
                   returned_apartments[returned_id]['json'],
                   returned_apartments[returned_id]['time_to_work']))

    added_ids.append(returned_apartments[returned_id]['id'])

logging.info('Checking removed entries')

for stored_id in stored_apartments.keys():
    if stored_id in returned_apartments.keys():
        continue

    logging.info(f'Removed apartment {stored_id}')

    cursor.execute('DELETE FROM apartments WHERE id=?', (stored_id, ))

    removed_ids.append(stored_apartments[stored_id]['id'])

logging.info('Checking updated entries')

for returned_id in returned_apartments.keys():
    if returned_id not in stored_apartments.keys():
        continue

    updated_ids_fields[returned_id] = []

    for key in returned_apartments[returned_id]:
        if key == 'json' or key == 'time_to_work' or key == 'published':
            continue

        if returned_apartments[returned_id][key] != stored_apartments[returned_id][key]:
            logging.info(f'Field {key} from {returned_id} changed from {stored_apartments[returned_id][key]} to {returned_apartments[returned_id][key]}')
            cursor.execute(f'UPDATE apartments SET {key}=? WHERE id=?', (returned_apartments[returned_id][key], returned_id))

            updated_ids_fields[returned_id].append(key)

            if key == 'coordinates':
                cursor.execute(f'UPDATE apartments SET time_to_work=NULL WHERE id=?', (returned_id,))

logging.info('Filling missing time to work')

cursor.execute('SELECT id, coordinates FROM apartments WHERE time_to_work IS NULL AND coordinates IS NOT NULL')
res = cursor.fetchall()

for row in res:
    time_to_work = None
    request_params = REQUEST_PARAMS
    request_params['origins'] = row[1]

    logging.info(f'Requesting time to work for ID {row[0]}')

    output = requests.get(API_URL, params=request_params)

    json_d = json.loads(output.text)

    try:
        time_to_work = json_d['rows'][0]['elements'][0]['duration']['value']
    except:
        logging.error(f"ERROR: No good value found ?! ID: {row[0]}, Coordinates: {row[1]}")
        conn.commit()
        raise Exception('Unable to get distance, please check')

    cursor.execute('UPDATE apartments SET time_to_work=? WHERE id=?', (time_to_work, row[0]))

apartments = {}

cursor.execute('''SELECT 
                  id, coordinates, published, floor, rooms,
                  size, postcode, price, url, json, time_to_work
                  FROM apartments''')

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

    apartments[row[0]] = apt

body_new = ''

for new_id in added_ids:
    body_new = body_new + apt_dict_to_str(apartments[new_id]) + '<br><br>'

body_removed = ''

for removed_id in removed_ids:
    body_removed = body_removed + apt_dict_to_str(stored_apartments[removed_id]) + '<br><br>'

body_updated = ''

for updated_id in updated_ids_fields.keys():
    if len(updated_ids_fields[updated_id]) == 0:
        continue

    body_updated = body_updated + apt_dict_to_str(apartments[updated_id]) + '<br>'

    for field in updated_ids_fields[updated_id]:
        body_updated = body_updated + f'Field {field} changed from {stored_apartments[updated_id][field]} to {returned_apartments[updated_id][field]}' + '<br>'

    body_updated = body_updated + '<br>'

body = ''

if body_new != '':
    body = body + '<h1>New apartments:</h1>' + body_new

if body_updated != '':
    body = body + '<h1>Updated apartments:</h1>' + body_updated

if body_removed != '':
    body = body + '<h1>Removed apartments:</h1>' + body_removed

if body != '':
    msg = MIMEText(body, 'html')
    msg['Subject'] = 'Updates in Willhaben'
    msg['From'] = ''
    msg['To'] = ''
    with smtplib.SMTP_SSL('smtp.gmail.com', 465) as smtp_server:
        smtp_server.login(msg['From'], '')
        smtp_server.sendmail(msg['From'], msg['To'], msg.as_string())

conn.commit()
conn.close()
